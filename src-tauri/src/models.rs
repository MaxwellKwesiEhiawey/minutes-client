use serde::{Deserialize, Serialize};

/// Lifecycle status of a meeting record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MeetingStatus {
    /// Actively capturing audio.
    Recording,
    /// Stopped cleanly by the user.
    Completed,
    /// App died/closed while still recording; recovered on next launch.
    Interrupted,
}

impl MeetingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MeetingStatus::Recording => "recording",
            MeetingStatus::Completed => "completed",
            MeetingStatus::Interrupted => "interrupted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: i64,
    pub meeting_id: String,
    pub seq: i64,
    pub text: String,
    pub created_at: String,
    /// Raw diarization label (e.g. `SPEAKER_0`), or a resolved name once known.
    #[serde(default)]
    pub speaker_label: Option<String>,
    /// Human-friendly speaker name after voice/calendar mapping. `None` = unknown.
    #[serde(default)]
    pub speaker_name: Option<String>,
    /// Segment start offset within the meeting, in milliseconds.
    #[serde(default)]
    pub start_ms: Option<i64>,
    /// Segment end offset within the meeting, in milliseconds.
    #[serde(default)]
    pub end_ms: Option<i64>,
}

/// A meeting plus a small amount of derived metadata for list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingListItem {
    #[serde(flatten)]
    pub meeting: Meeting,
    pub segment_count: i64,
    pub has_summary: bool,
}

/// A search result: a meeting list item plus an optional transcript snippet
/// (present when the query matched inside the transcript).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSearchHit {
    #[serde(flatten)]
    pub item: MeetingListItem,
    pub snippet: Option<String>,
}

/// Full detail returned to the UI for a single meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingDetail {
    pub meeting: Meeting,
    pub segments: Vec<Segment>,
    pub summary: Option<Summary>,
}

// ---- AI summary schema (must mirror the product spec exactly) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyTopic {
    pub topic: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub text: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub task: String,
    pub assignee: Option<String>,
    pub due: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryContent {
    pub title: String,
    pub executive_summary: String,
    pub key_topics: Vec<KeyTopic>,
    pub decisions: Vec<Decision>,
    pub action_items: Vec<ActionItem>,
    pub open_questions: Vec<String>,
}

/// A persisted summary: the structured content plus bookkeeping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub meeting_id: String,
    pub content: SummaryContent,
    pub model: String,
    pub created_at: String,
}

/// Kind of audio input device exposed by the OS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioDeviceKind {
    /// System-output loopback / monitor source (meeting audio).
    Loopback,
    /// Microphone or headset input.
    Microphone,
    /// Could not classify; still usable.
    Unknown,
}

/// An audio input device with a human-readable label for the Settings UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInputDevice {
    /// Exact device name passed to cpal when capturing.
    pub name: String,
    pub kind: AudioDeviceKind,
    /// Display label, e.g. "[System audio] Monitor of …".
    pub label: String,
}

/// Device list returned to the frontend, including platform hints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevicesResponse {
    pub platform: String,
    pub devices: Vec<AudioInputDevice>,
    pub has_loopback: bool,
}
