//! Per-device server credentials.
//!
//! Every install used to send the same bearer token — the one baked in at CI
//! build time. A secret compiled into a distributed binary is extractable by
//! anyone holding that binary, and because every copy presented the same value
//! the server could not attribute a request, revoke one machine, or cap what a
//! single install spent on its paid upstreams.
//!
//! The embedded token is now only a *bootstrap* credential: it registers this
//! install with `POST /v1/devices/register` and does nothing else. The server
//! returns a device token that is stored in the OS credential store and used
//! for every request afterwards.
//!
//! Registration happens lazily on the first request that needs a token rather
//! than at startup, so a machine that is offline (or a server that is briefly
//! down) fails the individual action instead of the whole launch.

use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::json;

use crate::error::CategorizedError;
use crate::settings::Settings;

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    device_id: String,
    device_token: String,
}

/// In-process copy of the device token, so the common path does not hit the OS
/// credential store on every request (which can prompt, and is slow on macOS).
///
/// A `tokio` mutex rather than a `std` one because it is deliberately held
/// across the registration request: two concurrent callers on a fresh install
/// would otherwise both register, leaving an orphan row on the server and one
/// of the two tokens unreachable.
fn cache() -> &'static tokio::sync::Mutex<Option<String>> {
    static CACHE: OnceLock<tokio::sync::Mutex<Option<String>>> = OnceLock::new();
    CACHE.get_or_init(|| tokio::sync::Mutex::new(None))
}

/// The token to authenticate with, registering this install if needed.
pub async fn auth_token(
    client: &reqwest::Client,
    settings: &Settings,
) -> Result<String, CategorizedError> {
    let mut cached = cache().lock().await;
    if let Some(token) = cached.as_ref() {
        return Ok(token.clone());
    }

    match crate::secrets::get_device_token() {
        Ok(Some(token)) if !token.trim().is_empty() => {
            *cached = Some(token.clone());
            return Ok(token);
        }
        Ok(_) => {}
        // A credential store that cannot be read is not proof the device is
        // unregistered, but there is nothing else to go on; registering again
        // is recoverable (an orphan row), whereas failing here is not.
        Err(e) => tracing::warn!("failed to read device token from OS credential store: {e}"),
    }

    let token = register(client, settings).await?;
    *cached = Some(token.clone());
    Ok(token)
}

/// Discard the current device credentials and register again.
///
/// Called when the server rejects a device token that had been working. The
/// usual cause is the registry being restored from a backup or otherwise lost;
/// re-registering turns what would be a permanent failure for every client into
/// a single retried request.
pub async fn reregister(
    client: &reqwest::Client,
    settings: &Settings,
) -> Result<String, CategorizedError> {
    let mut cached = cache().lock().await;
    *cached = None;
    if let Err(e) = crate::secrets::clear_device_credentials() {
        tracing::warn!("failed to clear device credentials: {e}");
    }
    let token = register(client, settings).await?;
    *cached = Some(token.clone());
    Ok(token)
}

async fn register(
    client: &reqwest::Client,
    settings: &Settings,
) -> Result<String, CategorizedError> {
    let bootstrap = settings.server_token().ok_or_else(|| {
        CategorizedError::coded(
            "error.serverTokenMissing",
            "Minutes server token is not configured",
        )
    })?;

    let registered = fetch_registration(client, &settings.server_url(), &bootstrap).await?;

    // A token that cannot be persisted still works for this run, but the next
    // launch would register again and orphan this row — worth a warning.
    if let Err(e) = crate::secrets::set_device_token(&registered.device_token) {
        tracing::warn!("failed to store device token in OS credential store: {e}");
    }
    if let Err(e) = crate::secrets::set_device_id(&registered.device_id) {
        tracing::warn!("failed to store device id in OS credential store: {e}");
    }

    tracing::info!("registered this device with the Minutes server");
    Ok(registered.device_token)
}

/// Ask the server for a device identity.
///
/// Split out from [`register`] so the request, the status mapping and the
/// parsing can be tested without a `Settings` or the OS credential store —
/// exercising them through `register` would write real credentials into the
/// developer's keychain, and would behave differently on CI, where there is no
/// unlocked keyring at all.
async fn fetch_registration(
    client: &reqwest::Client,
    server_url: &str,
    bootstrap: &str,
) -> Result<RegisterResponse, CategorizedError> {
    // Never send the bootstrap token over cleartext to a remote host.
    crate::settings::validate_server_url(server_url)?;

    let url = format!(
        "{}/v1/devices/register",
        server_url.trim().trim_end_matches('/')
    );
    let body = json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "app_version": env!("CARGO_PKG_VERSION"),
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {bootstrap}"))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("cannot reach Minutes server to register this device: {e}"))?;

    let status = resp.status();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(CategorizedError::coded(
            "error.serverRejectedToken",
            "Minutes server rejected the access token",
        ));
    }
    if status.as_u16() == 404 {
        // A server predating device registration. Say so plainly rather than
        // reporting a generic failure that looks like a network problem.
        return Err(CategorizedError::from(
            "This Minutes server does not support device registration. Ask IT to update it."
                .to_string(),
        ));
    }
    if !status.is_success() {
        return Err(CategorizedError::from(format!(
            "device registration failed ({status})"
        )));
    }

    resp.json()
        .await
        .map_err(|e| format!("invalid device registration response: {e}").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// What the stub server saw. Assertions cover the request as well as the
    /// response, because "did we actually send the bootstrap token, to the
    /// right path, with the right body" is half of what this code has to get
    /// right.
    struct Captured {
        head: String,
        body: String,
    }

    struct Stub {
        url: String,
        seen: Arc<Mutex<Vec<Captured>>>,
    }

    impl Stub {
        fn requests(&self) -> usize {
            self.seen.lock().unwrap().len()
        }
        fn last(&self) -> (String, String) {
            let seen = self.seen.lock().unwrap();
            let c = seen.last().expect("no request reached the stub server");
            (c.head.clone(), c.body.clone())
        }
    }

    /// Minimal HTTP/1.1 server answering every request with `status` and
    /// `body`. Runs the real `reqwest` client through the real code path, so
    /// status mapping and parsing are tested for what they do rather than
    /// against a mock of themselves.
    async fn stub(status: u16, body: &'static str) -> Stub {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);

        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let sink = Arc::clone(&sink);
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    let mut buf: Vec<u8> = Vec::new();
                    let mut chunk = [0u8; 4096];
                    // Read until the end of the headers, then however many
                    // more bytes content-length promises.
                    let head_end = loop {
                        match socket.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                        if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            break at + 4;
                        }
                        if buf.len() > 1 << 20 {
                            return;
                        }
                    };
                    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
                    let len: usize = head
                        .to_lowercase()
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:"))
                        .and_then(|v| v.trim().parse().ok())
                        .unwrap_or(0);
                    while buf.len() < head_end + len {
                        match socket.read(&mut chunk).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => buf.extend_from_slice(&chunk[..n]),
                        }
                    }
                    sink.lock().unwrap().push(Captured {
                        head,
                        body: String::from_utf8_lossy(&buf[head_end..]).into_owned(),
                    });

                    let response = format!(
                        "HTTP/1.1 {status} STATUS\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.flush().await;
                });
            }
        });

        Stub {
            url: format!("http://{addr}"),
            seen,
        }
    }

    #[tokio::test]
    async fn registers_and_reports_what_it_sent() {
        let stub = stub(
            201,
            r#"{"device_id":"11111111-2222-3333-4444-555555555555","device_token":"dsk_abc123"}"#,
        )
        .await;

        let registered = fetch_registration(&reqwest::Client::new(), &stub.url, "bootstrap-tok")
            .await
            .expect("registration should succeed");

        assert_eq!(registered.device_id, "11111111-2222-3333-4444-555555555555");
        assert_eq!(registered.device_token, "dsk_abc123");

        let (head, body) = stub.last();
        assert!(
            head.starts_with("POST /v1/devices/register "),
            "unexpected request line: {head}"
        );
        assert!(
            head.contains("authorization: Bearer bootstrap-tok")
                || head.contains("Authorization: Bearer bootstrap-tok"),
            "bootstrap token was not forwarded: {head}"
        );
        let sent: serde_json::Value = serde_json::from_str(&body).expect("body should be JSON");
        assert_eq!(sent["platform"], std::env::consts::OS);
        assert_eq!(sent["arch"], std::env::consts::ARCH);
        assert_eq!(sent["app_version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn a_rejected_bootstrap_token_is_reported_as_such() {
        for status in [401u16, 403] {
            let stub = stub(status, "{}").await;
            let err = fetch_registration(&reqwest::Client::new(), &stub.url, "nope")
                .await
                .expect_err("{status} must fail");
            assert_eq!(
                err.code,
                Some("error.serverRejectedToken"),
                "{status} should carry the token-rejected code"
            );
        }
    }

    /// The bootstrap token is what registration presents, so 401 and 403 both
    /// mean "this build's shared token was refused" here. The revoked/unknown
    /// distinction lives on the *device* token paths — the summary call and the
    /// live stream — not on this one.
    #[tokio::test]
    async fn registration_treats_both_refusals_as_a_rejected_bootstrap_token() {
        for status in [401u16, 403] {
            let stub = stub(status, "{}").await;
            let err = fetch_registration(&reqwest::Client::new(), &stub.url, "nope")
                .await
                .expect_err("{status} must fail");
            assert_eq!(err.code, Some("error.serverRejectedToken"));
        }
    }

    #[tokio::test]
    async fn a_server_without_the_endpoint_says_so() {
        let stub = stub(404, "{}").await;
        let err = fetch_registration(&reqwest::Client::new(), &stub.url, "tok")
            .await
            .expect_err("404 must fail");
        // The point of this branch: an out-of-date server must not read as a
        // network problem, which is what a generic failure would look like.
        assert!(
            err.message.contains("does not support device registration"),
            "unhelpful 404 message: {}",
            err.message
        );
        assert_eq!(err.code, None);
    }

    #[tokio::test]
    async fn a_server_error_is_not_mistaken_for_an_auth_failure() {
        let stub = stub(500, "{}").await;
        let err = fetch_registration(&reqwest::Client::new(), &stub.url, "tok")
            .await
            .expect_err("500 must fail");
        assert_ne!(err.code, Some("error.serverRejectedToken"));
        assert!(
            err.message.contains("device registration failed"),
            "unexpected message: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn a_malformed_body_fails_cleanly() {
        let stub = stub(201, "not json at all").await;
        let err = fetch_registration(&reqwest::Client::new(), &stub.url, "tok")
            .await
            .expect_err("a malformed body must fail");
        assert!(
            err.message.contains("invalid device registration response"),
            "unexpected message: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn cleartext_to_a_remote_host_is_refused() {
        // Same guard the transcribe and summarize paths rely on: the bootstrap
        // token must not go out over plain HTTP to anything but loopback.
        //
        // The message is asserted, not merely that it failed: an unreachable
        // host would also produce an error, and the two must not be confused —
        // this has to be refused by validation, before any request is built.
        let err = fetch_registration(&reqwest::Client::new(), "http://example.com", "tok")
            .await
            .expect_err("cleartext to a remote host must be refused");
        assert!(
            err.message.contains("insecure server URL"),
            "should fail validation, not the request: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn a_loopback_http_server_is_allowed() {
        // The other half of the rule above: local development over plain http
        // must keep working, so the guard cannot simply reject every http URL.
        let stub = stub(201, r#"{"device_id":"id","device_token":"dsk_x"}"#).await;
        assert!(stub.url.starts_with("http://127.0.0.1:"));
        fetch_registration(&reqwest::Client::new(), &stub.url, "tok")
            .await
            .expect("loopback http must be allowed");
        assert_eq!(stub.requests(), 1);
    }

    /// End-to-end against a real server — the only test here that proves the
    /// client and `desksec-server` actually agree on the wire format.
    ///
    /// Needs: `npm run dev` in desksec-server (on a build with
    /// `/v1/devices/register`), and `DESKSEC_TOKEN` / `DESKSEC_API_URL` in the
    /// client `.env` matching that server's `CLIENT_TOKENS`.
    #[tokio::test]
    #[ignore = "requires a running desksec-server and a bootstrap token in .env"]
    async fn registers_against_a_real_server() {
        crate::settings::reload_env_keys();
        let settings = Settings::default();
        let bootstrap = settings
            .server_token()
            .expect("DESKSEC_TOKEN must be set for this test");

        let registered =
            fetch_registration(&reqwest::Client::new(), &settings.server_url(), &bootstrap)
                .await
                .expect("registration should succeed against a live server");

        assert!(
            registered.device_token.starts_with("dsk_"),
            "unexpected token shape: {}",
            registered.device_token
        );
        assert!(!registered.device_id.is_empty());
    }
}
