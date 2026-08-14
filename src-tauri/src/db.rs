use crate::models::*;
use crate::secrets::DbKeyStatus;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};

/// Open (or create) the SQLite database and ensure the schema exists.
///
/// DATA-AT-REST: the database is encrypted with SQLCipher (see
/// `Cargo.toml`'s `rusqlite` dependency comment for why `bundled-sqlcipher`
/// is safe to enable alongside the vendored `minutes-core`'s plain
/// `bundled` request). The passphrase is a random, per-install value stored
/// in the OS credential store (macOS Keychain via `apple-native` / Windows
/// Credential Manager via `windows-native` / Linux Secret Service via
/// `async-secret-service` — see `secrets::get_or_create_db_key` and the
/// `keyring` dependency comment in `Cargo.toml`), so a copied-off
/// `desksec.db` file is unreadable without also compromising that OS-level
/// store. Every one of those stores persists across reboots, which the
/// passphrase must do to stay useful for the lifetime of the database file.
///
/// If the OS credential store is genuinely unavailable (e.g. some minimal
/// Linux setups with no Secret Service provider running), this falls back
/// to opening the database **without** encryption rather than failing
/// app startup outright, and logs a warning — at-rest protection then still
/// comes from OS-level full-disk encryption (FileVault / BitLocker / LUKS),
/// same as before this change, rather than bricking the app.
pub fn open(path: &std::path::Path) -> Result<OpenedDatabase> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    open_or_recover(path, crate::secrets::get_or_create_db_key)
}

/// An open database, plus whatever the caller needs to tell the user about how
/// it was obtained.
#[derive(Debug)]
pub struct OpenedDatabase {
    pub conn: Connection,
    /// Where an unreadable database was moved to, when one had to be. The
    /// caller surfaces this: losing every stored meeting is not something the
    /// user should have to find in a log file.
    pub quarantined: Option<std::path::PathBuf>,
}

/// Returned by [`open_with_key`] when an existing database file cannot be
/// decrypted with the key we hold.
///
/// This is deliberately a distinct type rather than a string: [`open_or_recover`]
/// must react to *only* this failure by quarantining the file, and must not
/// mistake an unrelated problem (a full disk, a permissions error, a genuinely
/// corrupt file) for a lost key and move a recoverable database aside.
#[derive(Debug, thiserror::Error)]
#[error("database exists but could not be decrypted with the configured key")]
pub struct UndecryptableDatabase;

/// Open the database, and if — and only if — it turns out to be encrypted
/// under a key we no longer have, move it aside and start a fresh one.
///
/// Losing the key means the contents are gone regardless: SQLCipher has no
/// recovery path without the passphrase. The choice is therefore between an
/// app that starts with an empty database and an app that cannot start at
/// all, and refusing to start strands the user with no way back. The old file
/// is preserved rather than deleted so that a key recovered later (a restored
/// keyring backup, say) can still be used against it by hand.
///
/// Quarantining is authorised by exactly one condition: a working credential
/// store reporting that no key exists ([`DbKeyStatus::Lost`]) while an
/// encrypted database is present. Every other way of failing to get a key is
/// inconclusive — the store may answer differently next launch — so when an
/// encrypted database is present this refuses to start rather than touch it.
/// Destroying a recoverable database is a far worse outcome than a startup
/// error the user can act on.
///
/// `key_for` is passed whether minting a new key is permitted, and is called a
/// second time after any quarantine. That second call is what keeps the
/// replacement database encrypted: minting is forbidden while the unreadable
/// database is still there, so the first call cannot produce a key, and reusing
/// that answer would leave the fresh database in plaintext for good.
fn open_or_recover(
    path: &std::path::Path,
    key_for: impl Fn(bool) -> DbKeyStatus,
) -> Result<OpenedDatabase> {
    let encrypted_exists = is_existing_encrypted_database(path)?;

    let key = match key_for(!encrypted_exists) {
        DbKeyStatus::Available(key) => Some(key),
        // Only reachable with an encrypted database present, since otherwise
        // minting was permitted. The key is definitively gone; `open_with_key`
        // will report `UndecryptableDatabase` and recovery takes over below.
        DbKeyStatus::Lost => None,
        DbKeyStatus::Unavailable(why) if encrypted_exists => {
            return Err(anyhow::anyhow!(
                "the database at {path:?} is encrypted, but the OS credential store could not be \
                 read ({why}). Refusing to continue: the key may well be readable again on the \
                 next launch, and treating this as a lost key would destroy the database. If a \
                 login keyring is locked, unlock it and start Minutes again."
            ));
        }
        DbKeyStatus::Unavailable(why) => {
            tracing::warn!(
                "could not obtain a database encryption key from the OS credential store ({why}); \
                 opening {path:?} unencrypted — at-rest protection relies on OS-level full-disk \
                 encryption only"
            );
            None
        }
    };

    match open_with_key(path, key.as_deref()) {
        Err(e) if e.downcast_ref::<UndecryptableDatabase>().is_some() => {
            let quarantined = quarantine_database(path)?;
            tracing::error!(
                "{path:?} is encrypted with a key that is no longer available; it has been \
                 preserved as {quarantined:?} and a new empty database created. Meetings stored \
                 in it cannot be recovered without the original key."
            );
            let key = match key_for(true) {
                DbKeyStatus::Available(key) => Some(key),
                DbKeyStatus::Lost => None,
                DbKeyStatus::Unavailable(why) => {
                    tracing::warn!(
                        "the replacement database at {path:?} will be unencrypted: the OS \
                         credential store could not be reached ({why})"
                    );
                    None
                }
            };
            Ok(OpenedDatabase {
                conn: open_with_key(path, key.as_deref())?,
                quarantined: Some(quarantined),
            })
        }
        other => Ok(OpenedDatabase {
            conn: other?,
            quarantined: None,
        }),
    }
}

/// Whether `path` holds an existing database that is encrypted — the state in
/// which a missing key is unrecoverable and a minted one would do damage.
///
/// Errors rather than guessing when the file is there but cannot be inspected.
fn is_existing_encrypted_database(path: &std::path::Path) -> Result<bool> {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e).with_context(|| format!("failed to inspect database at {path:?}")),
        // A zero-length file is what `File::create` leaves behind; there is no
        // header yet, so it is neither encrypted nor plaintext.
        Ok(m) if m.len() == 0 => Ok(false),
        Ok(_) => Ok(!is_plaintext_database(path)?),
    }
}

/// The `-wal` and `-shm` companion files SQLite keeps beside a database in WAL
/// mode (see the `journal_mode` pragma in [`init_schema`]).
///
/// These travel with the database or not at all. A write-ahead log holds
/// committed transactions that have not been checkpointed back into the main
/// file yet, so moving a database without its `-wal` silently loses them — and
/// leaving a stale `-wal` beside a *different* database of the same name invites
/// SQLite to replay it into that one.
///
/// Built by appending to the file name rather than with `Path::with_extension`,
/// which replaces everything after the last dot: that would turn `foo.db` into
/// `foo-wal`, and would eat part of the timestamp in a quarantined name (which
/// contains a dot), letting two quarantines in the same second collide.
fn sidecar_paths(path: &std::path::Path) -> [std::path::PathBuf; 2] {
    ["-wal", "-shm"].map(|suffix| {
        let mut name = path.to_path_buf().into_os_string();
        name.push(suffix);
        std::path::PathBuf::from(name)
    })
}

/// Rename a database file and bring its `-wal`/`-shm` sidecars along.
///
/// Failing to move the main file is the caller's problem to report; a sidecar
/// that will not move is logged and tolerated, because the alternative is
/// leaving a half-renamed database behind.
pub(crate) fn rename_with_sidecars(
    from: &std::path::Path,
    to: &std::path::Path,
) -> std::io::Result<()> {
    std::fs::rename(from, to)?;
    for (sidecar_from, sidecar_to) in sidecar_paths(from).into_iter().zip(sidecar_paths(to)) {
        if sidecar_from.exists() {
            if let Err(e) = std::fs::rename(&sidecar_from, &sidecar_to) {
                tracing::warn!("failed to move {sidecar_from:?} alongside the database: {e}");
            }
        }
    }
    Ok(())
}

/// Delete any `-wal`/`-shm` files left beside `path`, for when the database they
/// belonged to is gone and replaying them would corrupt its replacement.
fn remove_stale_sidecars(path: &std::path::Path) {
    for sidecar in sidecar_paths(path) {
        if sidecar.exists() {
            if let Err(e) = std::fs::remove_file(&sidecar) {
                tracing::warn!("failed to remove stale {sidecar:?}: {e}");
            }
        }
    }
}

/// Move an unopenable database out of the way, along with its `-wal` and
/// `-shm` sidecars, and return the path it was moved to.
///
/// The sidecars matter: leaving a stale write-ahead log next to a brand-new
/// database of the same name invites SQLite to replay it into the new file.
fn quarantine_database(path: &std::path::Path) -> Result<std::path::PathBuf> {
    // Colons are legal on Linux/macOS but not on Windows, and RFC3339 is full
    // of them.
    let stamp = crate::now_iso().replace(':', "-");
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "desksec.db".to_string());
    let target = path.with_file_name(format!("{file_name}.unreadable-{stamp}"));

    std::fs::rename(path, &target)
        .with_context(|| format!("failed to move undecryptable database aside to {target:?}"))?;

    for (from, to) in sidecar_paths(path).into_iter().zip(sidecar_paths(&target)) {
        if from.exists() {
            if let Err(e) = std::fs::rename(&from, &to) {
                tracing::warn!("failed to move stale sidecar {from:?} aside: {e}");
            }
        }
    }

    Ok(target)
}

/// Whether `path` is an unencrypted SQLite database, decided by its file
/// header rather than by trying to read it.
///
/// SQLite writes the fixed 16-byte string `SQLite format 3\0` at offset 0;
/// SQLCipher encrypts from the first byte, so an encrypted database starts
/// with ciphertext instead.
///
/// A missing or too-short file is definitively not plaintext — there is no
/// header, so there is nothing to migrate. Any *other* failure to read is an
/// error, never a `false`: callers act on `false` by treating the file as
/// encrypted, and "I could not open this file" must not become licence to
/// move a user's database aside. A denied read or an exhausted file-descriptor
/// table says nothing about the contents.
fn is_plaintext_database(path: &std::path::Path) -> Result<bool> {
    use std::io::Read;

    const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";

    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e).with_context(|| format!("failed to read the header of {path:?}")),
    };

    let mut header = [0u8; 16];
    match file.read_exact(&mut header) {
        Ok(()) => Ok(&header == SQLITE_HEADER),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(false),
        Err(e) => Err(e).with_context(|| format!("failed to read the header of {path:?}")),
    }
}

/// Does the actual work of `open()`, given the encryption key (or `None` to
/// open unencrypted) directly rather than fetching it from the OS keyring.
///
/// Split out specifically so tests can exercise the SQLCipher /
/// plaintext-migration logic below with a fixed, in-memory-only key: calling
/// through `open()` in a test would hit the *real* OS credential store,
/// which can pop a Keychain/Secret-Service access prompt during `cargo
/// test`, or simply hang/fail on a CI runner with no keyring backend at all.
fn open_with_key(path: &std::path::Path, key: Option<&str>) -> Result<Connection> {
    let Some(key) = key else {
        // No key available at all (no credential store, or the stored key is
        // gone and `secrets::get_or_create_db_key` refused to mint a
        // replacement). A file that already exists and is neither empty nor
        // plaintext SQLite is an encrypted database, and opening it unkeyed
        // would fail as "file is not a database" — report it as what it is so
        // `open_or_recover` can quarantine it rather than aborting startup.
        if is_existing_encrypted_database(path)? {
            return Err(UndecryptableDatabase.into());
        }
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        return Ok(conn);
    };

    // A database from before encryption was introduced needs the documented
    // `sqlcipher_export` migration. Identify it by its file header, which is
    // unambiguous. It must NOT be identified by probing with the key and
    // treating failure as "plaintext": a probe failure only says the key does
    // not decrypt this file, which is equally true of an encrypted database
    // whose key has been lost. Running `sqlcipher_export` over one of those
    // fails inside SQLCipher with "hmac check failed for pgno=1" — the crash
    // behind issue #5 — and, for a database small enough that the export
    // reads no pages, silently replaces it with an empty one instead.
    if is_plaintext_database(path)? {
        migrate_plaintext_to_encrypted(path, key)?;
    }

    let conn = Connection::open(path)?;
    conn.pragma_update(None, "key", key)
        .context("failed to set SQLCipher key on database connection")?;

    // `PRAGMA key` always "succeeds" even against the wrong key — SQLCipher
    // only discovers the mismatch on the first real page read, so probe with
    // a cheap one before handing the connection out. A brand-new (zero-byte)
    // file reads fine here and is keyed on first write.
    if conn
        .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        })
        .is_err()
    {
        return Err(UndecryptableDatabase.into());
    }

    init_schema(&conn)?;
    Ok(conn)
}

/// Encrypt an existing plaintext database file in place, following
/// SQLCipher's documented migration procedure: attach a new encrypted
/// database with the target key, copy everything over with
/// `sqlcipher_export()`, then swap the encrypted copy in.
/// See: https://www.zetetic.net/sqlcipher/sqlcipher-api/#sqlcipher_export
fn migrate_plaintext_to_encrypted(path: &std::path::Path, key: &str) -> Result<()> {
    let tmp_path = path.with_extension("db.encrypting");
    let _ = std::fs::remove_file(&tmp_path); // clean up any prior failed attempt

    let plain = Connection::open(path)
        .context("failed to open existing plaintext database for migration")?;
    // Both values are ours (a filesystem path we just built, and a
    // randomly-generated key) — never user/network input — but escape
    // single quotes defensively anyway since these are interpolated
    // directly into SQL text (`ATTACH`/`PRAGMA key` don't support bound
    // parameters).
    let escaped_tmp = tmp_path.to_string_lossy().replace('\'', "''");
    let escaped_key = key.replace('\'', "''");
    plain
        .execute_batch(&format!(
            "ATTACH DATABASE '{escaped_tmp}' AS encrypted KEY '{escaped_key}';
             SELECT sqlcipher_export('encrypted');
             DETACH DATABASE encrypted;"
        ))
        .context("failed to export plaintext database into an encrypted copy")?;
    drop(plain);

    std::fs::rename(&tmp_path, path)
        .context("failed to replace plaintext database with encrypted copy")?;

    // The swapped-in file is encrypted; any `-wal`/`-shm` still sitting beside it
    // belong to the plaintext database that was just replaced, and SQLCipher
    // cannot read them. Usually a no-op — the `drop(plain)` above closes that
    // connection, which checkpoints and removes its own sidecars — so this covers
    // the leftovers of an earlier unclean shutdown.
    remove_stale_sidecars(path);

    tracing::info!("migrated existing plaintext database to SQLCipher encryption at {path:?}");
    Ok(())
}

/// Create the schema and apply lightweight column migrations on an open
/// connection. Split out from [`open`] so tests can run it against an in-memory
/// database.
pub fn init_schema(conn: &Connection) -> Result<()> {
    // `PRAGMA journal_mode` returns the mode it settled on as a row, which
    // `execute_batch` discards — so a database that refuses WAL (a read-only or
    // network filesystem falls back to `delete`) reports success here. That is
    // tolerable, since every mode is correct for durability and only the sidecar
    // handling in `sidecar_paths` cares, and it is why that handling treats a
    // missing `-wal` as ordinary rather than surprising.
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS meetings (
            id          TEXT PRIMARY KEY,
            title       TEXT NOT NULL,
            status      TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            ended_at    TEXT
        );

        CREATE TABLE IF NOT EXISTS segments (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            meeting_id    TEXT NOT NULL,
            seq           INTEGER NOT NULL,
            text          TEXT NOT NULL,
            created_at    TEXT NOT NULL,
            speaker_label TEXT,
            speaker_name  TEXT,
            start_ms      INTEGER,
            end_ms        INTEGER,
            FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_segments_meeting ON segments(meeting_id, seq);

        CREATE TABLE IF NOT EXISTS summaries (
            meeting_id  TEXT PRIMARY KEY,
            content     TEXT NOT NULL,
            model       TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
        );
        "#,
    )?;

    // Migrate databases created before speaker diarization: add the new columns
    // if they are missing. SQLite has no "ADD COLUMN IF NOT EXISTS", so we ignore
    // the "duplicate column name" error when the column already exists.
    for stmt in [
        "ALTER TABLE segments ADD COLUMN speaker_label TEXT",
        "ALTER TABLE segments ADD COLUMN speaker_name TEXT",
        "ALTER TABLE segments ADD COLUMN start_ms INTEGER",
        "ALTER TABLE segments ADD COLUMN end_ms INTEGER",
    ] {
        if let Err(e) = conn.execute(stmt, []) {
            let msg = e.to_string();
            if !msg.contains("duplicate column name") {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// On launch, any meeting still flagged `recording` means the previous session
/// crashed or was force-closed mid-recording. Mark it `interrupted` so the UI
/// can surface it and the persisted segments remain available.
pub fn recover_interrupted(conn: &Connection) -> Result<usize> {
    let now = crate::now_iso();
    let n = conn.execute(
        "UPDATE meetings SET status = 'interrupted', ended_at = COALESCE(ended_at, ?1)
         WHERE status = 'recording'",
        params![now],
    )?;
    Ok(n)
}

pub fn create_meeting(conn: &Connection, title: &str) -> Result<Meeting> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = crate::now_iso();
    conn.execute(
        "INSERT INTO meetings (id, title, status, created_at, ended_at)
         VALUES (?1, ?2, 'recording', ?3, NULL)",
        params![id, title, created_at],
    )?;
    Ok(Meeting {
        id,
        title: title.to_string(),
        status: MeetingStatus::Recording.as_str().to_string(),
        created_at,
        ended_at: None,
    })
}

pub fn finalize_meeting(conn: &Connection, meeting_id: &str, status: MeetingStatus) -> Result<()> {
    let now = crate::now_iso();
    conn.execute(
        "UPDATE meetings SET status = ?1, ended_at = ?2 WHERE id = ?3",
        params![status.as_str(), now, meeting_id],
    )?;
    Ok(())
}

pub fn rename_meeting(conn: &Connection, meeting_id: &str, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE meetings SET title = ?1 WHERE id = ?2",
        params![title, meeting_id],
    )?;
    Ok(())
}

/// Persist a finalized transcript segment. Returns the stored row.
#[allow(dead_code)] // convenience wrapper; the pipeline uses `insert_segment_full`
pub fn insert_segment(conn: &Connection, meeting_id: &str, text: &str) -> Result<Segment> {
    insert_segment_full(conn, meeting_id, text, None, None, None, None)
}

/// Persist a finalized transcript segment with optional speaker + timing metadata.
pub fn insert_segment_full(
    conn: &Connection,
    meeting_id: &str,
    text: &str,
    speaker_label: Option<&str>,
    speaker_name: Option<&str>,
    start_ms: Option<i64>,
    end_ms: Option<i64>,
) -> Result<Segment> {
    let created_at = crate::now_iso();
    let seq: i64 = conn.query_row(
        "SELECT COALESCE(MAX(seq), -1) + 1 FROM segments WHERE meeting_id = ?1",
        params![meeting_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO segments (meeting_id, seq, text, created_at, speaker_label, speaker_name, start_ms, end_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![meeting_id, seq, text, created_at, speaker_label, speaker_name, start_ms, end_ms],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Segment {
        id,
        meeting_id: meeting_id.to_string(),
        seq,
        text: text.to_string(),
        created_at,
        speaker_label: speaker_label.map(str::to_string),
        speaker_name: speaker_name.map(str::to_string),
        start_ms,
        end_ms,
    })
}

pub fn get_meeting(conn: &Connection, meeting_id: &str) -> Result<Option<Meeting>> {
    let mut stmt =
        conn.prepare("SELECT id, title, status, created_at, ended_at FROM meetings WHERE id = ?1")?;
    let mut rows = stmt.query(params![meeting_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Meeting {
            id: row.get(0)?,
            title: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            ended_at: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_segments(conn: &Connection, meeting_id: &str) -> Result<Vec<Segment>> {
    let mut stmt = conn.prepare(
        "SELECT id, meeting_id, seq, text, created_at, speaker_label, speaker_name, start_ms, end_ms
         FROM segments
         WHERE meeting_id = ?1 ORDER BY seq ASC",
    )?;
    let rows = stmt.query_map(params![meeting_id], |row| {
        Ok(Segment {
            id: row.get(0)?,
            meeting_id: row.get(1)?,
            seq: row.get(2)?,
            text: row.get(3)?,
            created_at: row.get(4)?,
            speaker_label: row.get(5)?,
            speaker_name: row.get(6)?,
            start_ms: row.get(7)?,
            end_ms: row.get(8)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_meetings(conn: &Connection) -> Result<Vec<MeetingListItem>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.title, m.status, m.created_at, m.ended_at,
                (SELECT COUNT(*) FROM segments s WHERE s.meeting_id = m.id) AS seg_count,
                (SELECT COUNT(*) FROM summaries su WHERE su.meeting_id = m.id) AS has_sum
         FROM meetings m
         ORDER BY m.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let meeting = Meeting {
            id: row.get(0)?,
            title: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            ended_at: row.get(4)?,
        };
        let segment_count: i64 = row.get(5)?;
        let has_summary: i64 = row.get(6)?;
        Ok(MeetingListItem {
            meeting,
            segment_count,
            has_summary: has_summary > 0,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Escape SQLite `LIKE` wildcards in user input so a literal `%` or `_` in the
/// query isn't treated as a wildcard. Pairs with `ESCAPE '\'` in the statement.
fn escape_like(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    for ch in query.chars() {
        match ch {
            '\\' | '%' | '_' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Build a short snippet centered on the first case-insensitive match of `query`
/// within `text`, with ellipses when the snippet is trimmed at either end.
fn make_snippet(text: &str, query: &str) -> String {
    const WINDOW: usize = 64;
    let hay = text.to_lowercase();
    let needle = query.to_lowercase();
    let match_at = hay.find(&needle).unwrap_or(0);

    // Work on char boundaries: collect char indices once.
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    // Translate the byte offset of the match to a char index.
    let match_char = chars.iter().position(|(b, _)| *b >= match_at).unwrap_or(0);

    let start = match_char.saturating_sub(WINDOW);
    let end = (match_char + needle.chars().count() + WINDOW).min(chars.len());

    let slice: String = chars[start..end].iter().map(|(_, c)| *c).collect();
    let mut snippet = slice.trim().to_string();
    if start > 0 {
        snippet = format!("…{snippet}");
    }
    if end < chars.len() {
        snippet = format!("{snippet}…");
    }
    snippet
}

pub fn search_meetings(conn: &Connection, query: &str) -> Result<Vec<MeetingSearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let pattern = format!("%{}%", escape_like(trimmed));

    let mut stmt = conn.prepare(
        "SELECT m.id, m.title, m.status, m.created_at, m.ended_at,
                (SELECT COUNT(*) FROM segments s WHERE s.meeting_id = m.id) AS seg_count,
                (SELECT COUNT(*) FROM summaries su WHERE su.meeting_id = m.id) AS has_sum,
                (SELECT s2.text FROM segments s2
                   WHERE s2.meeting_id = m.id AND s2.text LIKE ?1 ESCAPE '\\'
                   ORDER BY s2.seq ASC LIMIT 1) AS snippet_text
         FROM meetings m
         WHERE m.title LIKE ?1 ESCAPE '\\'
            OR EXISTS (SELECT 1 FROM segments s3
                         WHERE s3.meeting_id = m.id AND s3.text LIKE ?1 ESCAPE '\\')
         ORDER BY m.created_at DESC",
    )?;

    let rows = stmt.query_map(params![pattern], |row| {
        let meeting = Meeting {
            id: row.get(0)?,
            title: row.get(1)?,
            status: row.get(2)?,
            created_at: row.get(3)?,
            ended_at: row.get(4)?,
        };
        let segment_count: i64 = row.get(5)?;
        let has_summary: i64 = row.get(6)?;
        let snippet_text: Option<String> = row.get(7)?;
        Ok(MeetingSearchHit {
            item: MeetingListItem {
                meeting,
                segment_count,
                has_summary: has_summary > 0,
            },
            snippet: snippet_text.map(|t| make_snippet(&t, trimmed)),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_summary(conn: &Connection, meeting_id: &str) -> Result<Option<Summary>> {
    let mut stmt = conn.prepare(
        "SELECT meeting_id, content, model, created_at FROM summaries WHERE meeting_id = ?1",
    )?;
    let mut rows = stmt.query(params![meeting_id])?;
    if let Some(row) = rows.next()? {
        let content_raw: String = row.get(1)?;
        let content: SummaryContent = serde_json::from_str(&content_raw)?;
        Ok(Some(Summary {
            meeting_id: row.get(0)?,
            content,
            model: row.get(2)?,
            created_at: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn upsert_summary(
    conn: &Connection,
    meeting_id: &str,
    content: &SummaryContent,
    model: &str,
) -> Result<Summary> {
    let created_at = crate::now_iso();
    let content_raw = serde_json::to_string(content)?;
    conn.execute(
        "INSERT INTO summaries (meeting_id, content, model, created_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(meeting_id) DO UPDATE SET content = ?2, model = ?3, created_at = ?4",
        params![meeting_id, content_raw, model, created_at],
    )?;
    Ok(Summary {
        meeting_id: meeting_id.to_string(),
        content: content.clone(),
        model: model.to_string(),
        created_at,
    })
}

pub fn delete_meeting(conn: &Connection, meeting_id: &str) -> Result<()> {
    // ON DELETE CASCADE handles segments + summary.
    conn.execute("DELETE FROM meetings WHERE id = ?1", params![meeting_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        // Match the on-disk PRAGMAs that affect behavior (FK cascade).
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn sample_summary() -> SummaryContent {
        SummaryContent {
            title: "Q3 Planning".to_string(),
            executive_summary: "We planned Q3.".to_string(),
            key_topics: vec![],
            decisions: vec![],
            action_items: vec![],
            open_questions: vec![],
        }
    }

    #[test]
    fn create_and_get_roundtrips() {
        let conn = mem();
        let m = create_meeting(&conn, "Standup").unwrap();
        let got = get_meeting(&conn, &m.id).unwrap().unwrap();
        assert_eq!(got.title, "Standup");
        assert_eq!(got.status, "recording");
        assert!(got.ended_at.is_none());
        assert!(get_meeting(&conn, "does-not-exist").unwrap().is_none());
    }

    #[test]
    fn segments_are_sequenced_and_ordered() {
        let conn = mem();
        let m = create_meeting(&conn, "Call").unwrap();
        let a = insert_segment_full(
            &conn,
            &m.id,
            "first",
            Some("SPEAKER_0"),
            None,
            Some(0),
            Some(10),
        )
        .unwrap();
        let b = insert_segment_full(&conn, &m.id, "second", None, None, None, None).unwrap();
        assert_eq!(a.seq, 0);
        assert_eq!(b.seq, 1);
        let segs = list_segments(&conn, &m.id).unwrap();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "first");
        assert_eq!(segs[0].speaker_label.as_deref(), Some("SPEAKER_0"));
        assert_eq!(segs[1].text, "second");
    }

    #[test]
    fn delete_cascades_to_segments_and_summary() {
        let conn = mem();
        let m = create_meeting(&conn, "Call").unwrap();
        insert_segment_full(&conn, &m.id, "hi", None, None, None, None).unwrap();
        upsert_summary(&conn, &m.id, &sample_summary(), "claude-x").unwrap();
        assert!(get_summary(&conn, &m.id).unwrap().is_some());

        delete_meeting(&conn, &m.id).unwrap();
        assert!(get_meeting(&conn, &m.id).unwrap().is_none());
        assert!(list_segments(&conn, &m.id).unwrap().is_empty());
        assert!(get_summary(&conn, &m.id).unwrap().is_none());
    }

    #[test]
    fn upsert_summary_replaces_existing() {
        let conn = mem();
        let m = create_meeting(&conn, "Call").unwrap();
        upsert_summary(&conn, &m.id, &sample_summary(), "model-a").unwrap();
        let mut updated = sample_summary();
        updated.title = "Renamed".to_string();
        upsert_summary(&conn, &m.id, &updated, "model-b").unwrap();
        let got = get_summary(&conn, &m.id).unwrap().unwrap();
        assert_eq!(got.content.title, "Renamed");
        assert_eq!(got.model, "model-b");
    }

    #[test]
    fn recover_interrupted_flags_stuck_recordings() {
        let conn = mem();
        let m = create_meeting(&conn, "Crashed").unwrap();
        let n = recover_interrupted(&conn).unwrap();
        assert_eq!(n, 1);
        let got = get_meeting(&conn, &m.id).unwrap().unwrap();
        assert_eq!(got.status, "interrupted");
        assert!(got.ended_at.is_some());
        // Idempotent: nothing left in `recording`.
        assert_eq!(recover_interrupted(&conn).unwrap(), 0);
    }

    #[test]
    fn search_matches_title_and_transcript_with_snippet() {
        let conn = mem();
        let m = create_meeting(&conn, "Budget review").unwrap();
        insert_segment_full(
            &conn,
            &m.id,
            "we discussed the marketing spend",
            None,
            None,
            None,
            None,
        )
        .unwrap();

        // Title match.
        let by_title = search_meetings(&conn, "budget").unwrap();
        assert_eq!(by_title.len(), 1);

        // Transcript match produces a snippet.
        let by_body = search_meetings(&conn, "marketing").unwrap();
        assert_eq!(by_body.len(), 1);
        assert!(by_body[0].snippet.as_deref().unwrap().contains("marketing"));

        // No match.
        assert!(search_meetings(&conn, "nonexistent-term")
            .unwrap()
            .is_empty());
        // Empty query returns nothing.
        assert!(search_meetings(&conn, "   ").unwrap().is_empty());
    }

    #[test]
    fn search_treats_like_wildcards_literally() {
        let conn = mem();
        let m = create_meeting(&conn, "plain title").unwrap();
        insert_segment_full(&conn, &m.id, "literal text only", None, None, None, None).unwrap();
        // A bare "%" must not match everything (it is escaped).
        assert!(search_meetings(&conn, "%").unwrap().is_empty());
        assert!(search_meetings(&conn, "_").unwrap().is_empty());
    }

    #[test]
    fn list_meetings_reports_counts() {
        let conn = mem();
        let m = create_meeting(&conn, "Call").unwrap();
        insert_segment_full(&conn, &m.id, "a", None, None, None, None).unwrap();
        insert_segment_full(&conn, &m.id, "b", None, None, None, None).unwrap();
        upsert_summary(&conn, &m.id, &sample_summary(), "model").unwrap();
        let items = list_meetings(&conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].segment_count, 2);
        assert!(items[0].has_summary);
    }

    // --- SQLCipher encryption (open_with_key / migrate_plaintext_to_encrypted) ---
    //
    // These use `open_with_key` directly (never `open`/`secrets::get_or_create_db_key`)
    // so they never touch the real OS credential store — see `open_with_key`'s doc
    // comment for why that matters for `cargo test` in CI.

    #[test]
    fn open_with_key_creates_a_fresh_encrypted_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fresh.db");

        let conn = open_with_key(&path, Some("test-key-one")).unwrap();
        create_meeting(&conn, "Encrypted from birth").unwrap();
        drop(conn);

        // Re-opening with the same key must see the data...
        let conn = open_with_key(&path, Some("test-key-one")).unwrap();
        assert_eq!(list_meetings(&conn).unwrap().len(), 1);
        drop(conn);

        // ...but a connection with no key must NOT be able to read it as
        // plain SQLite (the whole point of enabling `bundled-sqlcipher`).
        let unkeyed = Connection::open(&path).unwrap();
        let result = unkeyed.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        });
        assert!(
            result.is_err(),
            "expected reading an encrypted database without a key to fail"
        );
    }

    #[test]
    fn open_with_key_rejects_the_wrong_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keyed.db");

        let conn = open_with_key(&path, Some("correct-key")).unwrap();
        create_meeting(&conn, "Secret meeting").unwrap();
        drop(conn);

        // Opening with the wrong key should behave like opening a foreign,
        // undecryptable file: init_schema's first read fails.
        let reopened = open_with_key(&path, Some("wrong-key"));
        assert!(reopened.is_err());
    }

    #[test]
    fn wrong_key_is_reported_as_undecryptable_not_migrated_as_plaintext() {
        // Regression test for the crash in issue #5. An encrypted database
        // whose key has been lost is NOT a plaintext database: running the
        // `sqlcipher_export` migration over it fails deep inside SQLCipher
        // ("hmac check failed for pgno=1"), which used to propagate out of
        // the Tauri setup hook and abort the whole app at launch.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("orphaned.db");

        let conn = open_with_key(&path, Some("lost-key")).unwrap();
        create_meeting(&conn, "Meeting encrypted under a lost key").unwrap();
        drop(conn);

        let err = open_with_key(&path, Some("replacement-key")).unwrap_err();
        assert!(
            err.downcast_ref::<UndecryptableDatabase>().is_some(),
            "expected UndecryptableDatabase, got: {err:#}"
        );

        // The migration must not have run: the original file is untouched and
        // no half-written temp file is left behind.
        assert!(!path.with_extension("db.encrypting").exists());
        assert!(!is_plaintext_database(&path).unwrap());
    }

    #[test]
    fn plaintext_detection_reads_the_file_header() {
        let dir = tempfile::tempdir().unwrap();

        // A real plaintext SQLite database.
        let plain = dir.path().join("plain.db");
        drop(open_with_key(&plain, None).unwrap());
        assert!(is_plaintext_database(&plain).unwrap());

        // An encrypted one.
        let encrypted = dir.path().join("encrypted.db");
        drop(open_with_key(&encrypted, Some("k")).unwrap());
        assert!(!is_plaintext_database(&encrypted).unwrap());

        // A file that does not exist yet, and one too short to hold a header,
        // are both "not plaintext" — there is nothing to migrate.
        assert!(!is_plaintext_database(&dir.path().join("missing.db")).unwrap());
        let stub = dir.path().join("stub.db");
        std::fs::write(&stub, b"SQLite").unwrap();
        assert!(!is_plaintext_database(&stub).unwrap());
        // Nor is a zero-length file, which is what an interrupted first run
        // leaves behind.
        let empty = dir.path().join("empty.db");
        std::fs::write(&empty, b"").unwrap();
        assert!(!is_plaintext_database(&empty).unwrap());
        assert!(!is_existing_encrypted_database(&empty).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_file_is_an_error_not_an_assumption() {
        // Regression test: `is_plaintext_database` used to answer `false` on
        // any read failure, and `false` means "encrypted" to every caller —
        // which would send a perfectly good plaintext database down the
        // quarantine path on nothing more than a permissions error.
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("locked.db");
        drop(open_with_key(&path, None).unwrap());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Running as root defeats the permission bits entirely; skip rather
        // than assert something untrue.
        if std::fs::File::open(&path).is_ok() {
            return;
        }

        assert!(is_plaintext_database(&path).is_err());
        assert!(is_existing_encrypted_database(&path).is_err());
        // And the whole open must fail rather than quarantine the file.
        assert!(open_or_recover(&path, |_| DbKeyStatus::Available("k".into())).is_err());
        assert!(path.exists(), "an unreadable database must not be moved");

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    #[test]
    fn an_unavailable_credential_store_never_touches_an_encrypted_database() {
        // The most important test here. A locked GNOME keyring, a dismissed
        // macOS Keychain prompt, or a missing D-Bus session all fail to
        // produce a key while saying nothing about whether one exists. Acting
        // on that as though the key were lost would destroy a database that
        // opens perfectly well on the next launch.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        {
            let conn = open_with_key(&path, Some("real-key")).unwrap();
            create_meeting(&conn, "Still recoverable").unwrap();
        }
        let before = std::fs::read(&path).unwrap();

        let err = open_or_recover(&path, |_| {
            DbKeyStatus::Unavailable("keyring is locked".into())
        })
        .unwrap_err();
        assert!(
            format!("{err:#}").contains("keyring is locked"),
            "the underlying reason must reach the user: {err:#}"
        );

        // Untouched: not quarantined, not re-created, not modified.
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let quarantined = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains(".unreadable-"));
        assert!(
            !quarantined,
            "an inconclusive key lookup must not quarantine"
        );

        // And once the store comes back, the data is still there.
        let opened = open_or_recover(&path, |_| DbKeyStatus::Available("real-key".into())).unwrap();
        assert_eq!(list_meetings(&opened.conn).unwrap().len(), 1);
        assert!(opened.quarantined.is_none());
    }

    #[test]
    fn an_unavailable_credential_store_still_allows_a_first_run() {
        // The documented fallback: with no database to endanger, an
        // unreachable store means an unencrypted database rather than a
        // refusal to start.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        let opened =
            open_or_recover(&path, |_| DbKeyStatus::Unavailable("no d-bus".into())).unwrap();
        create_meeting(&opened.conn, "Unencrypted but usable").unwrap();
        drop(opened);

        assert!(is_plaintext_database(&path).unwrap());
    }

    #[test]
    fn open_or_recover_quarantines_an_undecryptable_database_and_starts_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        {
            let conn = open_with_key(&path, Some("lost-key")).unwrap();
            create_meeting(&conn, "Unrecoverable meeting").unwrap();
        }

        let opened =
            open_or_recover(&path, |_| DbKeyStatus::Available("replacement-key".into())).unwrap();
        // Usable, and empty — the old meeting is gone, not silently readable.
        assert_eq!(list_meetings(&opened.conn).unwrap().len(), 0);
        create_meeting(&opened.conn, "Fresh start").unwrap();
        assert_eq!(list_meetings(&opened.conn).unwrap().len(), 1);
        let quarantined_path = opened.quarantined.clone().expect("caller must be told");
        drop(opened);

        assert!(quarantined_path.exists());
        assert!(quarantined_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("desksec.db.unreadable-"));
    }

    #[test]
    fn quarantine_takes_the_wal_and_shm_sidecars_with_it() {
        // Tested against `quarantine_database` directly rather than through
        // `open_or_recover`: SQLite removes both sidecars when the probe
        // connection closes, so by the time recovery runs there is usually
        // nothing left to move and an end-to-end assertion would pass
        // vacuously. This code exists for the case where that cleanup did not
        // happen — a killed process, a crash mid-write — and leaving a stale
        // write-ahead log beside a brand-new database of the same name invites
        // SQLite to replay it into the new file.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");
        std::fs::write(&path, b"encrypted bytes").unwrap();
        std::fs::write(path.with_extension("db-wal"), b"stale wal").unwrap();
        std::fs::write(path.with_extension("db-shm"), b"stale shm").unwrap();

        let target = quarantine_database(&path).unwrap();

        assert!(!path.exists(), "the database itself must be moved");
        assert!(!path.with_extension("db-wal").exists());
        assert!(!path.with_extension("db-shm").exists());

        let moved = |suffix: &str| {
            let mut p = target.clone().into_os_string();
            p.push(suffix);
            std::path::PathBuf::from(p)
        };
        assert_eq!(std::fs::read(&target).unwrap(), b"encrypted bytes");
        assert_eq!(std::fs::read(moved("-wal")).unwrap(), b"stale wal");
        assert_eq!(std::fs::read(moved("-shm")).unwrap(), b"stale shm");
    }

    #[test]
    fn renaming_a_database_brings_its_sidecars_along() {
        // The legacy `parley.db` → `desksec.db` migration in `lib.rs` used a bare
        // `fs::rename`, which stranded the write-ahead log: a `-wal` holds
        // committed transactions not yet checkpointed into the main file, so
        // leaving it behind loses them, and leaving it beside a database of the
        // old name means nothing ever replays it.
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("parley.db");
        let to = dir.path().join("desksec.db");
        std::fs::write(&from, b"database").unwrap();
        std::fs::write(dir.path().join("parley.db-wal"), b"committed but unflushed").unwrap();
        std::fs::write(dir.path().join("parley.db-shm"), b"shared memory").unwrap();

        rename_with_sidecars(&from, &to).unwrap();

        assert!(!from.exists());
        assert!(!dir.path().join("parley.db-wal").exists());
        assert!(!dir.path().join("parley.db-shm").exists());
        assert_eq!(std::fs::read(&to).unwrap(), b"database");
        assert_eq!(
            std::fs::read(dir.path().join("desksec.db-wal")).unwrap(),
            b"committed but unflushed"
        );
        assert_eq!(
            std::fs::read(dir.path().join("desksec.db-shm")).unwrap(),
            b"shared memory"
        );
    }

    #[test]
    fn renaming_a_database_without_sidecars_is_not_an_error() {
        // The common case: a cleanly-closed database has no sidecars at all, so
        // their absence must not look like a failure.
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("parley.db");
        let to = dir.path().join("desksec.db");
        std::fs::write(&from, b"database").unwrap();

        rename_with_sidecars(&from, &to).unwrap();

        assert!(to.exists());
        assert!(!dir.path().join("desksec.db-wal").exists());
    }

    #[test]
    fn encrypting_an_existing_database_clears_the_plaintext_sidecars() {
        // After the swap, a `-wal` beside the file belongs to the plaintext
        // database that was just replaced. SQLCipher cannot read it, and letting
        // SQLite try to replay it into the encrypted file is how a working
        // database becomes an unopenable one.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        {
            let conn = Connection::open(&path).unwrap();
            init_schema(&conn).unwrap();
            create_meeting(&conn, "Plaintext meeting").unwrap();
        }
        // Stand in for what an unclean shutdown leaves behind; a clean close
        // removes these, which is why the production path is usually a no-op.
        std::fs::write(dir.path().join("desksec.db-wal"), b"stale plaintext wal").unwrap();
        std::fs::write(dir.path().join("desksec.db-shm"), b"stale plaintext shm").unwrap();

        migrate_plaintext_to_encrypted(&path, "new-key").unwrap();

        assert!(!dir.path().join("desksec.db-wal").exists());
        assert!(!dir.path().join("desksec.db-shm").exists());
        // And the migration itself still did its job.
        assert!(!is_plaintext_database(&path).unwrap());
        let conn = open_with_key(&path, Some("new-key")).unwrap();
        assert_eq!(list_meetings(&conn).unwrap().len(), 1);
    }

    #[test]
    fn open_or_recover_leaves_a_readable_database_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        {
            let conn = open_with_key(&path, Some("stable-key")).unwrap();
            create_meeting(&conn, "Keep me").unwrap();
        }

        let opened =
            open_or_recover(&path, |_| DbKeyStatus::Available("stable-key".into())).unwrap();
        let items = list_meetings(&opened.conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].meeting.title, "Keep me");
        assert!(opened.quarantined.is_none());
        drop(opened);

        let quarantined = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains(".unreadable-"));
        assert!(
            !quarantined,
            "a readable database must never be quarantined"
        );
    }

    #[test]
    fn open_or_recover_still_migrates_a_plaintext_database() {
        // The pre-encryption upgrade path must survive the recovery wrapper:
        // a plaintext database is migrated, never quarantined.
        //
        // The provider here mirrors the real one, which will not mint a key
        // while an *encrypted* database with no key is present. Handing back a
        // key unconditionally would hide the bug this guards: if a plaintext
        // database were reported as "an encrypted database exists", minting
        // would be refused, and the very users this migration exists for would
        // stay on an unencrypted database forever.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        {
            let conn = open_with_key(&path, None).unwrap();
            create_meeting(&conn, "From before encryption").unwrap();
        }

        let opened = open_or_recover(&path, |may_mint| {
            if may_mint {
                DbKeyStatus::Available("new-key".into())
            } else {
                DbKeyStatus::Lost
            }
        })
        .unwrap();
        let items = list_meetings(&opened.conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].meeting.title, "From before encryption");
        assert!(opened.quarantined.is_none());
        drop(opened);

        // Migrated, not quarantined, and genuinely encrypted afterwards.
        assert!(!is_plaintext_database(&path).unwrap());
        let quarantined = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains(".unreadable-"));
        assert!(
            !quarantined,
            "a plaintext database must be migrated, not quarantined"
        );
    }

    #[test]
    fn recovery_re_keys_the_replacement_database() {
        // `secrets::get_or_create_db_key` will not mint a replacement key
        // while an undecryptable database is still present, so the first key
        // lookup of a recovery reports `Lost`. Once the file has been
        // quarantined that objection is gone, and the replacement database
        // must be encrypted like any other first run — otherwise recovering
        // from a lost key silently downgrades the app to an unencrypted
        // database forever.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("desksec.db");

        {
            let conn = open_with_key(&path, Some("lost-key")).unwrap();
            create_meeting(&conn, "Unrecoverable").unwrap();
        }

        let opened = open_or_recover(&path, |may_mint| {
            if may_mint {
                DbKeyStatus::Available("minted-after-quarantine".into())
            } else {
                DbKeyStatus::Lost
            }
        })
        .unwrap();
        create_meeting(&opened.conn, "Fresh start").unwrap();
        assert!(opened.quarantined.is_some());
        drop(opened);

        assert!(
            !is_plaintext_database(&path).unwrap(),
            "the replacement database must be encrypted, not plaintext"
        );
        let reopened = open_with_key(&path, Some("minted-after-quarantine")).unwrap();
        assert_eq!(list_meetings(&reopened).unwrap().len(), 1);
    }

    #[test]
    fn open_with_key_migrates_an_existing_plaintext_database_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.db");

        // Simulate a pre-encryption install: a plain, unencrypted database
        // created the old way (no key at all).
        {
            let conn = open_with_key(&path, None).unwrap();
            create_meeting(&conn, "Pre-existing meeting").unwrap();
        }

        // Opening the same file with a key for the first time must migrate
        // it in place rather than treating it as corrupt, and must not lose
        // the existing meeting.
        let conn = open_with_key(&path, Some("new-key")).unwrap();
        let items = list_meetings(&conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].meeting.title, "Pre-existing meeting");
        drop(conn);

        // No leftover temp file from the `sqlcipher_export` swap.
        assert!(!path.with_extension("db.encrypting").exists());

        // The file on disk is now genuinely encrypted, not just re-saved as
        // plaintext: an unkeyed connection can no longer read it.
        let unkeyed = Connection::open(&path).unwrap();
        let result = unkeyed.query_row("SELECT count(*) FROM sqlite_master", [], |r| {
            r.get::<_, i64>(0)
        });
        assert!(result.is_err());
    }

    #[test]
    fn open_with_key_none_opens_plaintext_as_before() {
        // Covers the OS-keyring-unavailable fallback path in `open()`.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("unencrypted.db");

        let conn = open_with_key(&path, None).unwrap();
        create_meeting(&conn, "No key configured").unwrap();
        drop(conn);

        // A plain, keyless connection can read it straight away.
        let plain = Connection::open(&path).unwrap();
        let count: i64 = plain
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0))
            .unwrap();
        assert!(count > 0);
    }
}
