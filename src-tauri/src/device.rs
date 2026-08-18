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
    let server_url = settings.server_url();
    // Never send the bootstrap token over cleartext to a remote host.
    crate::settings::validate_server_url(&server_url)?;

    let bootstrap = settings.server_token().ok_or_else(|| {
        CategorizedError::coded(
            "error.serverTokenMissing",
            "Minutes server token is not configured",
        )
    })?;

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

    let registered: RegisterResponse = resp
        .json()
        .await
        .map_err(|e| format!("invalid device registration response: {e}"))?;

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
