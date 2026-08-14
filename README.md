# Minutes — bot-free meeting recorder with on-device transcription

A cross-platform (**macOS / Linux / Windows**) desktop app that records meetings
**without a meeting bot**, transcribes speech **entirely on-device**, shows a
**live transcript** with **speaker labels**, generates a structured **AI
summary**, and lets you **delete**, **share**, or **export** meeting artifacts.

Minutes is built by combining a clean desktop experience (React + a
Tauri v2 / Rust core, SQLite storage, server-backed summarization) onto the
[silverstein/minutes](https://github.com/silverstein/minutes) engine (local
Whisper transcription via whisper.cpp, plus pyannote-rs speaker diarization). The
full Minutes monorepo is vendored under [`minutes/`](minutes/), so its CLI, MCP
server, relationship graph, and vault tooling remain available.

---

## What changed vs. a cloud recorder

| Stage | How Minutes does it |
| --- | --- |
| **Speech → text (default)** | **Live stream** to Minutes server (`WS /v1/transcribe/stream` → Deepgram Live). Interim captions + finalized segments in the UI. |
| **Speech → text (optional)** | **On-device Whisper** in Settings — no STT vendor key on the client; download a model once. |
| **Speaker labels** | **Deepgram diarization** when online, or **on-device pyannote-rs** when using Whisper. |
| **Storage** | Local **SQLite** (`desksec.db`) — every final segment is persisted the instant it arrives (crash-safe). Optionally mirrored to `~/meetings/*.md` for the Minutes ecosystem. The database is **not application-encrypted**; at-rest confidentiality relies on **OS full-disk encryption** (FileVault / BitLocker / LUKS) — enable it on any device that records confidential meetings. |
| **Transcript → summary** | The finished transcript is POSTed to the **Minutes server** (`/v1/summarize` → Fireworks AI), which returns a structured JSON summary. On demand. |

```
Mic / system audio ──► [online: Minutes server → Deepgram] or [on-device: whisper.cpp] ──► SQLite ──► [Generate summary] ──► Minutes server ──► Fireworks AI
```

The Minutes server URL + bearer token are provisioned at **CI build time**
(`DESKSEC_API_URL` variable + `DESKSEC_TOKEN` secret → compiled into the binary).
Embedded values always win over local `.env`, `settings.json`, and keychain.
On first launch the token is copied into the OS credential store. For local dev
without a CI build, use `.env` or Settings → Advanced.
Remote servers must use `https://` (plain `http://` is allowed only for
`localhost`), so the token and transcript are never sent in cleartext.

---

## Features

1. **Start / stop recording** — one click; local transcription begins immediately.
   No server token is required to record (transcription is offline).
2. **Live transcript** — on-device Whisper emits interim (partial) text every few
   seconds and persists a final, speaker-attributed segment every chunk.
3. **Speaker labels** — pyannote-rs attributes each segment to a speaker
   (`SPEAKER_0`, …). Toggle off in Settings if not needed.
4. **Crash recovery** — a meeting still flagged `recording` at launch is marked
   `interrupted`; already-persisted segments reload normally.
5. **AI summary (on demand)** — send the transcript (with speaker labels) to the
   Minutes server for a structured summary: title, executive summary, key topics,
   decisions, action items, open questions. The meeting adopts the AI title.
6. **Delete / Share / Export** — copy the summary or raw transcript, or export to
   Markdown (`.md`) or Word (`.docx`).
7. **Markdown vault bridge** — completed meetings are mirrored to `~/meetings/` so
   the vendored Minutes CLI / MCP / relationship graph can read them.

### Quality-of-life

- **On-device model picker** — `tiny` / `base` / `small` (default) / `medium` /
  `large-v3`, downloaded on first run (Metal-accelerated on macOS).
- **Language** — auto-detect or an ISO 639-1 code (full Whisper language range).
- **Audio capture source picker** — system-audio loopback + microphone, with
  optional mic mixing.
- **Summary instructions** — global default plus optional per-meeting instructions.
- **Theme + reading comfort** — text size, line spacing, high contrast, reduced motion.

---

## Architecture

```
.
├─ src/                      # React + TypeScript UI (Minutes)
│  ├─ App.tsx                # orchestrates state + live events
│  ├─ api.ts                 # invoke() wrappers + event listeners
│  └─ components/            # Sidebar, MeetingView, SummaryView, ShareModal, SettingsModal
├─ src-tauri/src/            # Rust core (desktop)
│  ├─ lib.rs                 # Tauri builder, DB init, crash recovery, command registry
│  ├─ local_transcribe.rs    # on-device Whisper + diarization adapter (wraps minutes-core)
│  ├─ recorder.rs            # capture → on-device transcription pipeline (chunked partial/final)
│  ├─ vault_export.rs        # markdown export bridge to ~/meetings
│  ├─ db.rs                  # SQLite storage (meetings, segments+speakers, summaries)
│  ├─ summary.rs             # summary client (→ server /v1/summarize)
│  ├─ audio.rs               # cpal capture, device classification, dual-stream mix
│  ├─ secrets.rs             # OS credential store for the server token
│  ├─ docx_export.rs         # Word (.docx) export
│  └─ settings.rs            # local settings + server config resolution
└─ minutes/                  # vendored silverstein/minutes monorepo (engine + CLI + MCP + graph)
   └─ crates/core            # minutes-core: whisper.cpp transcription, pyannote-rs diarization
```

### Recording pipeline

1. **Capture** — `cpal` opens the selected input; optional mic + system-audio mixing.
2. **Transcribe (local)** — every `chunk_secs` a 16 kHz WAV is transcribed by
   whisper.cpp via `minutes_core::transcribe::transcribe`; every `partial_secs`
   an interim pass emits live text. All on-device.
3. **Diarize (local)** — when enabled, `minutes_core::diarize::diarize` labels the
   speaker for each segment.
4. **Persist** — each final segment (text + speaker + timing) is written to SQLite
   immediately and emitted to the UI.
5. **Summarize (on demand)** — the user clicks **Generate summary**; the transcript
   is POSTed to the Minutes server and the structured result is stored locally.

---

## Prerequisites

- **Rust** (stable) and **Node.js 18+**
- **CMake** (whisper.cpp build) and **LLVM/Clang** (bindgen for whisper-rs / pyannote-rs)
- Tauri v2 system deps: https://tauri.app/start/prerequisites/
- Linux also needs WebKitGTK, libasound (ALSA), libpipewire, `libsecret-1-dev`, and
  `ffmpeg` (recommended for non-English audio).
- macOS: Xcode Command Line Tools (for Metal). On macOS 26+, set
  `export CXXFLAGS="-I$(xcrun --show-sdk-path)/usr/include/c++/v1"` before building.

## Setup & run (dev)

Two processes: the **desksec-server** (holds the Fireworks AI key for summarization)
and the **Minutes desktop** (this repo).

```bash
# 1. Start the summarization server (separate repo)
cd desksec-server && npm install && cp .env.example .env
#    edit: FIREWORKS_API_KEY + a CLIENT_TOKENS value
npm run dev          # http://localhost:8787

# 2. Start the Minutes desktop (repo root)
npm install
cp .env.example .env
#    edit .env: DESKSEC_TOKEN must match one of the server's CLIENT_TOKENS
npm run tauri dev
```

On first launch, open **Settings → Transcription** and click **Download model**
to fetch the `small` Whisper model (and, if speaker labels are on, the diarization
models). Then start recording.

> `.env` is gitignored; never commit real tokens.
> macOS prompts for **microphone permission** on first record.

## Build (release installers)

```bash
npm run tauri build
```

GPU acceleration: macOS release builds enable **Metal** by default. Windows/Linux
CUDA/ROCm/Vulkan are opt-in whisper-rs features (see `minutes/README.md`).

### Public downloads (GitHub Releases)

CI builds installers for **macOS (Apple Silicon + Intel), Linux, and Windows**.
Pushing a version tag publishes them as a public **GitHub Release** anyone can
download from the repo’s Releases page:

```bash
# On the commit you want to ship (usually main after merge):
git tag -a v0.1.0 -m "Minutes v0.1.0"
git push origin v0.1.0
```

That triggers `.github/workflows/build-desktop.yml`: all platform builds must
succeed, then a non-draft Release is created with `.dmg` / `.AppImage` / `.deb` /
`.rpm` / `.msi` / `.exe` assets. Tag format must match `v*` (e.g. `v0.1.0`).

See [CONTRIBUTING.md](CONTRIBUTING.md#cutting-a-public-release) for the full
checklist (signing, server URL variable, verifying assets).

---

## Configuration

| Setting | Default | Notes |
| --- | --- | --- |
| Whisper model | `small` | on-device accuracy/size tradeoff |
| Identify speakers | on | on-device pyannote-rs diarization |
| Export to `~/meetings` | on | markdown mirror for the Minutes ecosystem |
| Spoken language | auto-detect | ISO 639-1 code, or auto |
| Chunk length | `8 s` | final-segment interval |
| Partial interval | `4 s` | interim-text interval; `0` disables |
| Server URL | (CI embed) | Baked in at build from `DESKSEC_API_URL`; locked in Settings |
| Access token | (CI embed) | Baked in at build from `DESKSEC_TOKEN`; stored in OS keychain |
| Summary model | `accounts/fireworks/models/gpt-oss-120b` | summarization override (Advanced) |
| Summary language | match transcript | language the AI summary is written in |
| Summary instructions | — | optional prompt applied to every summary |

Theme and reading-comfort preferences are stored in local browser storage.

---

## The vendored Minutes engine

`minutes/` is the full [silverstein/minutes](https://github.com/silverstein/minutes)
monorepo (MIT). Minutes depends on its `minutes-core` crate for on-device
transcription and diarization, and mirrors finished meetings to `~/meetings/` so
the Minutes CLI, MCP server, and relationship graph work against Minutes
recordings. See [`minutes/README.md`](minutes/README.md) for those surfaces.
