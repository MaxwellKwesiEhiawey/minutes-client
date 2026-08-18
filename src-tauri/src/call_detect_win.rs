//! Which applications are using the microphone, on Windows.
//!
//! macOS answers this with a CoreAudio helper that reports the PIDs holding an
//! input stream. Windows has no equivalent worth shelling out to, but it does
//! keep the answer in the registry: the Capability Access Manager records, per
//! application, when microphone access started and stopped. An entry whose
//! `LastUsedTimeStop` is zero is using the microphone *right now* — this is the
//! same bookkeeping behind the "an app is using your microphone" indicator in
//! the system tray.
//!
//! That is a better fit than the macOS shape, not a worse one: the store names
//! the application directly, so there is no PID to map back through the process
//! tree the way `native_app_candidate_process_pids` has to.
//!
//! Everything here except [`consent_entries`] is pure and compiled on every
//! platform, so the parsing is exercised by `cargo test` on a Mac even though
//! the registry read can only run on Windows.

// Live only on Windows, but compiled everywhere so `cargo test` on any machine
// exercises the parsing — the registry read is the sole part that cannot be.
#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

use std::collections::HashSet;

/// Where the Capability Access Manager records microphone use, under HKCU.
#[cfg(target_os = "windows")]
const MIC_CONSENT_KEY: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone";

/// Desktop apps are recorded one level deeper than packaged ones.
#[cfg(target_os = "windows")]
const NON_PACKAGED: &str = "NonPackaged";

/// One row of the consent store: the subkey name, and its `LastUsedTimeStop`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentEntry {
    pub key: String,
    /// Zero means access has started and not yet stopped.
    pub last_used_time_stop: u64,
}

/// The application name a consent-store subkey refers to.
///
/// Desktop entries are a full path with `#` where a separator would be, e.g.
/// `C:#Program Files#Zoom#bin#Zoom.exe`. Packaged entries are a package family
/// name, e.g. `MSTeams_8wekyb3d8bbwe`, whose suffix is a publisher hash and
/// carries no meaning for matching.
pub fn app_name_from_consent_key(key: &str) -> Option<String> {
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    if let Some(last) = key.rsplit('#').next() {
        if last.to_lowercase().ends_with(".exe") {
            return Some(last.to_string());
        }
    }
    // Packaged app: strip the publisher hash after the final underscore.
    if !key.contains('#') {
        let name = key.rsplit_once('_').map(|(n, _)| n).unwrap_or(key);
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// The applications currently holding the microphone.
pub fn capturing_apps(entries: &[ConsentEntry]) -> HashSet<String> {
    entries
        .iter()
        .filter(|e| e.last_used_time_stop == 0)
        .filter_map(|e| app_name_from_consent_key(&e.key))
        .collect()
}

/// Read the consent store. Returns `None` if it cannot be read at all, which
/// callers must treat as "unknown", not as "nothing is capturing" — the
/// difference decides whether a missed read looks like a call ending.
#[cfg(target_os = "windows")]
pub fn consent_entries() -> Option<Vec<ConsentEntry>> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let root = hkcu
        .open_subkey_with_flags(MIC_CONSENT_KEY, KEY_READ)
        .ok()?;

    let mut entries = Vec::new();
    let mut read_into = |parent: &RegKey| {
        for name in parent.enum_keys().flatten() {
            if name == NON_PACKAGED {
                continue;
            }
            if let Ok(sub) = parent.open_subkey_with_flags(&name, KEY_READ) {
                // A missing value means the app has permission but has never
                // used the microphone; treat it as not capturing rather than
                // as capturing-since-forever.
                let stop: u64 = sub.get_value("LastUsedTimeStop").unwrap_or(1);
                entries.push(ConsentEntry {
                    key: name,
                    last_used_time_stop: stop,
                });
            }
        }
    };

    read_into(&root);
    if let Ok(non_packaged) = root.open_subkey_with_flags(NON_PACKAGED, KEY_READ) {
        read_into(&non_packaged);
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, stop: u64) -> ConsentEntry {
        ConsentEntry {
            key: key.into(),
            last_used_time_stop: stop,
        }
    }

    #[test]
    fn reads_the_executable_out_of_a_desktop_entry() {
        assert_eq!(
            app_name_from_consent_key(r"C:#Program Files#Zoom#bin#Zoom.exe").as_deref(),
            Some("Zoom.exe")
        );
        assert_eq!(
            app_name_from_consent_key(r"C:#Users#sam#AppData#Local#slack#slack.exe").as_deref(),
            Some("slack.exe")
        );
    }

    #[test]
    fn strips_the_publisher_hash_from_a_packaged_entry() {
        // The suffix identifies the publisher, not the app, and differs between
        // Store builds of the same product.
        assert_eq!(
            app_name_from_consent_key("MSTeams_8wekyb3d8bbwe").as_deref(),
            Some("MSTeams")
        );
        assert_eq!(
            app_name_from_consent_key("Microsoft.SkypeApp_kzf8qxf38zg5c").as_deref(),
            Some("Microsoft.SkypeApp")
        );
    }

    #[test]
    fn ignores_entries_that_name_nothing_usable() {
        assert_eq!(app_name_from_consent_key(""), None);
        assert_eq!(app_name_from_consent_key("   "), None);
        // A path-shaped key whose last segment is not a program.
        assert_eq!(app_name_from_consent_key("C:#Program Files#Zoom#bin"), None);
    }

    #[test]
    fn only_a_zero_stop_time_counts_as_in_use() {
        let entries = vec![
            entry(r"C:#Program Files#Zoom#bin#Zoom.exe", 0),
            entry(
                r"C:#Users#sam#AppData#Local#slack#slack.exe",
                133_700_000_000_000_000,
            ),
            entry("MSTeams_8wekyb3d8bbwe", 0),
        ];
        let live = capturing_apps(&entries);
        assert!(
            live.contains("Zoom.exe"),
            "a zero stop time means in use now"
        );
        assert!(live.contains("MSTeams"));
        assert!(
            !live.contains("slack.exe"),
            "a real stop timestamp means the app has finished with the mic"
        );
        assert_eq!(live.len(), 2);
    }

    #[test]
    fn nothing_capturing_is_an_empty_set_not_an_error() {
        let entries = vec![entry(r"C:#Program Files#Zoom#bin#Zoom.exe", 1)];
        assert!(capturing_apps(&entries).is_empty());
    }
}
