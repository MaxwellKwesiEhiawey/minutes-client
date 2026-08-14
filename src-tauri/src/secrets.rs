use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "com.desksec.app";
const LEGACY_SERVICE: &str = "app.parley.desktop";
const ACCOUNT_TOKEN: &str = "desksec-server-token";
const LEGACY_ACCOUNT_TOKEN: &str = "parley-server-token";
const ACCOUNT_API_URL: &str = "desksec-server-url";
const LEGACY_ACCOUNT_API_URL: &str = "parley-server-url";
const ACCOUNT_DB_KEY: &str = "desksec-db-key";

fn entry(service: &str, account: &str) -> Result<Entry> {
    Entry::new(service, account).context("failed to open OS credential store")
}

fn get_secret(account: &str, legacy_service: &str, legacy_account: &str) -> Result<Option<String>> {
    match entry(SERVICE, account)?.get_password() {
        Ok(value) => return Ok(Some(value)),
        Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(e).context("failed to read from OS credential store"),
    }

    match entry(legacy_service, legacy_account)?.get_password() {
        Ok(value) => {
            let _ = set_secret(account, &value);
            let _ = entry(legacy_service, legacy_account)?.delete_credential();
            Ok(Some(value))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e).context("failed to read legacy OS credential store"),
    }
}

fn set_secret(account: &str, value: &str) -> Result<()> {
    entry(SERVICE, account)?
        .set_password(value.trim())
        .context("failed to store in OS credential store")
}

/// Read the Minutes bearer token from the OS credential store.
pub fn get_token() -> Result<Option<String>> {
    get_secret(ACCOUNT_TOKEN, LEGACY_SERVICE, LEGACY_ACCOUNT_TOKEN)
}

/// Persist the Minutes bearer token in the OS credential store.
pub fn set_token(token: &str) -> Result<()> {
    set_secret(ACCOUNT_TOKEN, token)
}

/// Read the Minutes server URL from the OS credential store.
pub fn get_api_url() -> Result<Option<String>> {
    get_secret(ACCOUNT_API_URL, LEGACY_SERVICE, LEGACY_ACCOUNT_API_URL)
}

/// Persist the Minutes server URL in the OS credential store.
pub fn set_api_url(url: &str) -> Result<()> {
    set_secret(ACCOUNT_API_URL, url)
}

/// Get the local database's SQLCipher encryption passphrase from the OS
/// credential store, generating and persisting a new random one on first
/// run. Used by `db::open()` to key the `desksec.db` connection.
///
/// This deliberately does NOT fall back to the legacy `app.parley.desktop`
/// service the way tokens/URLs do — this key never existed under that name,
/// since encryption-at-rest is a new feature, not a migrated one.
///
/// The passphrase is two concatenated UUID v4s (64 hex characters): `uuid`
/// sources its randomness from `getrandom` (a CSPRNG), and SQLCipher runs
/// the passphrase through PBKDF2 internally, so it doesn't need to already
/// be a raw fixed-size key — an arbitrary-length high-entropy string is the
/// documented, supported input to `PRAGMA key`.
/// What the OS credential store had to say when asked for the database key.
///
/// The distinction between [`Lost`](DbKeyStatus::Lost) and
/// [`Unavailable`](DbKeyStatus::Unavailable) is the whole point of this type,
/// and collapsing the two is a data-loss bug. `Lost` is a definitive answer
/// from a working store: there is no key, and there never will be. Only that
/// answer justifies `db::open` moving an unreadable database aside.
/// `Unavailable` means the store could not be consulted — a locked GNOME
/// keyring, a dismissed macOS Keychain prompt, no D-Bus session over SSH — and
/// says nothing at all about whether a key exists. Those resolve on the next
/// launch, so the database must be left strictly alone.
pub enum DbKeyStatus {
    Available(String),
    Lost,
    Unavailable(String),
}

/// Get the local database's SQLCipher passphrase from the OS credential store,
/// generating and persisting a new random one when `may_mint` allows it.
///
/// `may_mint` must be false whenever an encrypted database is already present:
/// a freshly minted key cannot open that file, and storing it would overwrite
/// the slot the original belongs in, destroying the last chance of reading the
/// database from a restored keyring backup. `db::open_or_recover` passes true
/// only for a first run, a plaintext database awaiting migration, or after an
/// unreadable database has been quarantined.
pub fn get_or_create_db_key(may_mint: bool) -> DbKeyStatus {
    let e = match entry(SERVICE, ACCOUNT_DB_KEY) {
        Ok(e) => e,
        Err(err) => return DbKeyStatus::Unavailable(format!("{err:#}")),
    };

    match e.get_password() {
        Ok(value) => DbKeyStatus::Available(value),
        Err(keyring::Error::NoEntry) if !may_mint => DbKeyStatus::Lost,
        Err(keyring::Error::NoEntry) => {
            let key = format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            );
            match e.set_password(&key) {
                Ok(()) => DbKeyStatus::Available(key),
                // A key we could not persist is worse than no key: it would
                // encrypt the database this run and be gone the next.
                Err(err) => DbKeyStatus::Unavailable(format!(
                    "failed to store database key in OS credential store: {err}"
                )),
            }
        }
        Err(err) => DbKeyStatus::Unavailable(format!(
            "failed to read database key from OS credential store: {err}"
        )),
    }
}
