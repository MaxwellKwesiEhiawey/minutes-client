//! Which processes are recording audio, on Linux.
//!
//! Both audio stacks in common use answer this through `pactl`: PulseAudio
//! natively, and PipeWire through `pipewire-pulse`, which nearly every PipeWire
//! desktop ships for compatibility. One parser therefore covers both, where
//! talking to PipeWire directly would cover only the newer half.
//!
//! `pactl list source-outputs` lists every stream reading from a source, and
//! each carries the PID of the application that opened it. That is the same
//! shape macOS produces, so the matching logic — walking the process tree to
//! catch an app's audio helper — is shared rather than reimplemented.
//!
//! Everything except [`capturing_pids`] is pure and compiled on every platform,
//! so the parsing is tested wherever `cargo test` runs. Running `pactl` is the
//! only part that needs an actual Linux desktop.

#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use std::collections::HashSet;

/// PIDs with a live, uncorked capture stream.
///
/// A corked stream is one the application has paused: Zoom keeps a source
/// output open while muted or idle in the tray, so counting corked streams
/// would report a meeting whenever Zoom was merely running — the exact
/// false positive that makes a process-only check useless.
pub fn capturing_pids_from_pactl(text: &str) -> HashSet<u32> {
    let mut pids = HashSet::new();
    let mut corked = false;
    let mut pid: Option<u32> = None;

    let flush = |pids: &mut HashSet<u32>, corked: bool, pid: Option<u32>| {
        if !corked {
            if let Some(p) = pid {
                pids.insert(p);
            }
        }
    };

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Source Output #") {
            // A new block: commit whatever the previous one described.
            flush(&mut pids, corked, pid);
            corked = false;
            pid = None;
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Corked:") {
            corked = value.trim().eq_ignore_ascii_case("yes");
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("application.process.id") {
            pid = value
                .trim_start()
                .trim_start_matches('=')
                .trim()
                .trim_matches('"')
                .parse()
                .ok();
        }
    }
    flush(&mut pids, corked, pid);
    pids
}

/// Ask the audio server which processes are capturing.
///
/// `None` means the question could not be asked — no `pactl`, or no session bus
/// — which is not the same as "nothing is recording"; the caller must not read
/// a failure as an empty room.
#[cfg(target_os = "linux")]
pub fn capturing_pids() -> Option<HashSet<u32>> {
    let output = std::process::Command::new("pactl")
        .args(["list", "source-outputs"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(capturing_pids_from_pactl(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shaped after real `pactl list source-outputs` output, trimmed to the
    /// fields this parser reads.
    const TWO_STREAMS: &str = r#"
Source Output #412
	Driver: PipeWire
	Owner Module: n/a
	Client: 88
	Source: 46
	Sample Specification: s16le 1ch 48000Hz
	Corked: no
	Mute: no
	Properties:
		media.name = "capture"
		application.name = "Zoom"
		application.process.id = "4242"
		application.process.binary = "zoom"

Source Output #415
	Driver: PipeWire
	Client: 91
	Source: 46
	Corked: yes
	Mute: no
	Properties:
		application.name = "Firefox"
		application.process.id = "5150"
		application.process.binary = "firefox"
"#;

    #[test]
    fn finds_the_pid_of_a_live_capture_stream() {
        let pids = capturing_pids_from_pactl(TWO_STREAMS);
        assert!(pids.contains(&4242), "Zoom is capturing and must be seen");
    }

    #[test]
    fn a_corked_stream_is_not_a_meeting() {
        // Apps hold a paused source output open while idle; counting those
        // would prompt whenever the app was merely running.
        let pids = capturing_pids_from_pactl(TWO_STREAMS);
        assert!(!pids.contains(&5150), "a corked stream is not recording");
        assert_eq!(pids.len(), 1);
    }

    #[test]
    fn nothing_recording_yields_nothing() {
        assert!(capturing_pids_from_pactl("").is_empty());
        assert!(capturing_pids_from_pactl("Failure: No such entity").is_empty());
    }

    #[test]
    fn the_last_block_is_not_dropped() {
        // The parser commits a block when the next one starts, so the final
        // stream is only counted if the end of input flushes too.
        let one = "Source Output #1\n\tCorked: no\n\t\tapplication.process.id = \"77\"\n";
        assert!(capturing_pids_from_pactl(one).contains(&77));
    }

    #[test]
    fn a_block_without_a_pid_is_skipped_without_taking_the_next_one_with_it() {
        let text = concat!(
            "Source Output #1\n\tCorked: no\n\t\tapplication.name = \"anonymous\"\n",
            "Source Output #2\n\tCorked: no\n\t\tapplication.process.id = \"99\"\n",
        );
        let pids = capturing_pids_from_pactl(text);
        assert_eq!(pids.len(), 1);
        assert!(pids.contains(&99));
    }

    #[test]
    fn corked_state_does_not_leak_between_blocks() {
        // Corked is reset per block; without that, one paused stream would
        // silence every stream listed after it.
        let text = concat!(
            "Source Output #1\n\tCorked: yes\n\t\tapplication.process.id = \"11\"\n",
            "Source Output #2\n\tCorked: no\n\t\tapplication.process.id = \"22\"\n",
        );
        let pids = capturing_pids_from_pactl(text);
        assert!(!pids.contains(&11));
        assert!(pids.contains(&22));
    }
}
