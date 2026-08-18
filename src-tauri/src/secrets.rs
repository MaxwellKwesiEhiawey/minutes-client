use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "com.minutes.app";
/// Credential-store services this app has used before, newest first.
///
/// The bundle identifier is part of the keychain service name, so renaming the
/// identifier orphans every secret stored under the old one. Each rename adds
/// an entry here; reads fall through the list and migrate the first hit
/// forward, so a user who skips a version still recovers.
const LEGACY_SERVICE_DESKSEC: &str = "com.desksec.app";
const LEGACY_SERVICE_PARLEY: &str = "app.parley.desktop";
const ACCOUNT_TOKEN: &str = "desksec-server-token";
const LEGACY_ACCOUNT_TOKEN: &str = "parley-server-token";
const ACCOUNT_API_URL: &str = "desksec-server-url";
const LEGACY_ACCOUNT_API_URL: &str = "parley-server-url";
const ACCOUNT_DB_KEY: &str = "desksec-db-key";
// Per-device credentials issued by the server (see `crate::device`). Kept in
// slots of their own rather than reusing ACCOUNT_TOKEN, because
// `settings::apply_embedded_server_config` rewrites that slot from the
// CI-embedded value on *every* launch — a provisioned token stored there would
// be silently destroyed on the next start.
const ACCOUNT_DEVICE_TOKEN: &str = "desksec-device-token";
const ACCOUNT_DEVICE_ID: &str = "desksec-device-id";

fn entry(service: &str, account: &str) -> Result<Entry> {
    Entry::new(service, account).context("failed to open OS credential store")
}

/// Read `account`, falling back through `legacy` — a list of (service, account)
/// pairs in newest-first order — and migrating the first value found.
///
/// A legacy read that fails for a reason other than "no such entry" is not
/// fatal: a locked keyring should leave the remaining candidates to be tried,
/// and the next launch to retry, rather than turning into a hard error.
fn get_secret(account: &str, legacy: &[(&str, &str)]) -> Result<Option<String>> {
    match entry(SERVICE, account)?.get_password() {
        Ok(value) => return Ok(Some(value)),
        Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(e).context("failed to read from OS credential store"),
    }

    for (legacy_service, legacy_account) in legacy {
        let Ok(legacy_entry) = entry(legacy_service, legacy_account) else {
            continue;
        };
        match legacy_entry.get_password() {
            Ok(value) => {
                let _ = set_secret(account, &value);
                let _ = legacy_entry.delete_credential();
                return Ok(Some(value));
            }
            Err(keyring::Error::NoEntry) => continue,
            Err(e) => {
                tracing::warn!("could not read legacy credential {legacy_service}: {e}");
                continue;
            }
        }
    }
    Ok(None)
}

fn set_secret(account: &str, value: &str) -> Result<()> {
    entry(SERVICE, account)?
        .set_password(value.trim())
        .context("failed to store in OS credential store")
}

/// Read the Minutes bearer token from the OS credential store.
pub fn get_token() -> Result<Option<String>> {
    get_secret(
        ACCOUNT_TOKEN,
        &[
            (LEGACY_SERVICE_DESKSEC, ACCOUNT_TOKEN),
            (LEGACY_SERVICE_PARLEY, LEGACY_ACCOUNT_TOKEN),
        ],
    )
}

/// Persist the Minutes bearer token in the OS credential store.
pub fn set_token(token: &str) -> Result<()> {
    set_secret(ACCOUNT_TOKEN, token)
}

/// Read this install's device token from the OS credential store.
///
/// Falls back to the `com.desksec.app` service, but not to the parley-era one:
/// device registration postdates that rename, so no value can exist there.
pub fn get_device_token() -> Result<Option<String>> {
    get_secret(
        ACCOUNT_DEVICE_TOKEN,
        &[(LEGACY_SERVICE_DESKSEC, ACCOUNT_DEVICE_TOKEN)],
    )
}

/// Persist this install's device token in the OS credential store.
pub fn set_device_token(token: &str) -> Result<()> {
    set_secret(ACCOUNT_DEVICE_TOKEN, token)
}

/// Read this install's server-assigned device id.
pub fn get_device_id() -> Result<Option<String>> {
    get_secret(
        ACCOUNT_DEVICE_ID,
        &[(LEGACY_SERVICE_DESKSEC, ACCOUNT_DEVICE_ID)],
    )
}

/// Persist this install's server-assigned device id.
pub fn set_device_id(id: &str) -> Result<()> {
    set_secret(ACCOUNT_DEVICE_ID, id)
}

/// Forget the device credentials so the next request registers afresh.
///
/// Called when the server rejects the device token — the registry may have been
/// restored from a backup, or the device revoked and later re-approved.
pub fn clear_device_credentials() -> Result<()> {
    for account in [ACCOUNT_DEVICE_TOKEN, ACCOUNT_DEVICE_ID] {
        match entry(SERVICE, account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e).context("failed to clear device credentials"),
        }
    }
    Ok(())
}

/// Read the Minutes server URL from the OS credential store.
pub fn get_api_url() -> Result<Option<String>> {
    get_secret(
        ACCOUNT_API_URL,
        &[
            (LEGACY_SERVICE_DESKSEC, ACCOUNT_API_URL),
            (LEGACY_SERVICE_PARLEY, LEGACY_ACCOUNT_API_URL),
        ],
    )
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

/// Move a database key stored under a previous service name into the current
/// one. Returns the key when there was one to move.
///
/// The copy is written before the original is removed, and a failed write
/// abandons the migration with the original intact: a key that exists in
/// neither place is an unopenable database.
fn migrate_db_key_from(legacy_service: &str, current: &Entry) -> Result<Option<String>, String> {
    let legacy = match Entry::new(legacy_service, ACCOUNT_DB_KEY) {
        Ok(entry) => entry,
        Err(e) => return Err(format!("could not open {legacy_service}: {e}")),
    };
    let value = match legacy.get_password() {
        Ok(value) => value,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(format!("could not read {legacy_service}: {e}")),
    };
    if let Err(e) = current.set_password(&value) {
        return Err(format!("could not carry the database key forward: {e}"));
    }
    // Only now is it safe to drop the old copy.
    let _ = legacy.delete_credential();
    tracing::info!("migrated the database key from {legacy_service}");
    Ok(Some(value))
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
        Err(keyring::Error::NoEntry) => {
            // The identifier is part of the service name, so a rename hides the
            // key rather than deleting it. Recover it before concluding
            // anything: `Lost` makes `db::open_or_recover` move the database
            // aside and start empty, which would turn a rename into data loss
            // for every existing user.
            match migrate_db_key_from(LEGACY_SERVICE_DESKSEC, &e) {
                Ok(Some(value)) => return DbKeyStatus::Available(value),
                Ok(None) => {}
                // Unreadable is not the same as absent — say so, so the
                // database is left alone until the store can be read.
                Err(err) => return DbKeyStatus::Unavailable(err),
            }
            if !may_mint {
                return DbKeyStatus::Lost;
            }
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
