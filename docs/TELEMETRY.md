# Minutes Telemetry — Event Schema (v1)

> # Metadata only. Never meeting content.
>
> Minutes telemetry **never** carries transcript text, summary text, meeting
> titles, participant or speaker names, audio, file paths, search queries, or
> the name of a detected call app. Not truncated. Not hashed. Not sampled.
> This is enforced mechanically by an attribute-key allowlist plus tests that
> fail the build if a content-shaped key is ever added — see
> [§1 Privacy rules](#1-privacy-rules-read-first). It applies to the disk
> spool as well: the files Minutes writes to disk while offline are the same
> content-free payloads that would have gone over the wire.

Status: **live for internal releases.** CI embeds the Grafana write token in
the internal-only, VPN-distributed installer. Metadata-only telemetry is active
in those builds, and the existing Settings → Privacy control can disable it.
This document is the contract for what the app may ever report. Any change to
it needs a privacy review.

## 0. Settled decisions

These were agreed with the stakeholder and are implemented. They are recorded
here so they do not get relitigated by accident.

| Decision | Status |
| --- | --- |
| Telemetry is **ON by default**, with a one-click opt-out in Settings → Privacy. | Implemented |
| **Metadata only, never meeting content.** | Implemented + enforced by tests |
| Emission is **asynchronous**, never synchronous. Nothing may block the recording pipeline. | Implemented |
| **Disk spool + retry with backoff**, so an offline app does not lose events. | Implemented ([§2.4](#24-disk-spool-and-retry)) |
| **Richer metadata**: OS version, CPU architecture, core count, build channel, per-launch session ID. | Implemented ([§2](#2-transport)) |
| Performance events (startup, summary, transcription, model download). | Defined in [§5.5](#55-performance); three of four wired |
| Destination is **Grafana Cloud**, stack `1549080`, EU West 2 gateway. OTLP/HTTP **JSON confirmed working** (HTTP 204 against the live endpoint) — protobuf is *not* required. | Implemented ([§3](#3-configuration)) |
| **Retention: 12 months.** | Agreed ([§6](#6-retention--access)) |
| High-cardinality identifiers must **not** be resource attributes. | Implemented + enforced by test ([§2.2](#22-cardinality-rule)) |

## 1. Privacy rules (read first)

Minutes records private meetings. That is the product. Telemetry therefore
follows one rule with no exceptions:

> **Metadata only. Never content.**

Concretely, the following are **NEVER collected**, in any form — not
truncated, not hashed, not "just the first N characters":

- Transcript text, partial transcripts, or summary text
- Meeting titles (including AI-generated ones)
- Participant or speaker names, or speaker labels
- Audio, in any representation
- File paths, file names, export destinations
- Device names (microphones, loopback devices)
- Hostnames, usernames, machine IDs, serial numbers, MAC addresses, IPs
- Email addresses or any account identifier
- Server URLs, tokens, or any setting whose value is free text
  (e.g. summary instructions)
- Error **messages** (they can embed URLs and environment details) — only
  the error **category** is sent
- The name of the app that triggered a call-detection prompt (only
  "a call prompt was shown" is reported, never *which* app caused it)

What **is** collected: event names, counts, coarse buckets, closed category
sets, boolean feature flags, the app version, the OS name and coarse version,
the CPU architecture and core count, the build channel, a per-launch session
ID, and a pseudonymous install ID.

**Retention: 12 months.** See [§6](#6-retention--access).

Enforcement is mechanical, not aspirational: every event attribute key must
be on the allowlist in `src-tauri/src/telemetry.rs`
(`ALLOWED_ATTR_KEYS`). Non-allowlisted attributes are dropped before they
reach the send queue, and a unit test fails if a content-shaped key (one
containing `title`, `text`, `name`, `path`, …) is ever added to the
allowlist.

Resource attributes (the per-install description, §2) go through a second,
separate allowlist, `ALLOWED_RESOURCE_ATTR_KEYS`, and the two identifiers
through a third, `IDENTITY_ATTR_KEYS`. None of them is caller-supplied —
`build_export_payload` builds the exact lists — and tests assert the payload
and the allowlists agree, so a new one cannot appear by accident. The split
between the two is the cardinality rule in
[§2.2](#22-cardinality-rule), not a stylistic choice.

That guard has teeth. The summary-latency attribute is named
`summarize_duration_bucket`, not `summary_duration_bucket`, because the
allowlist test reserves the `summary_` prefix for content fields
(`summary_text`, `summary_instructions`). The guard was left alone and the
attribute was renamed.

### Identity

- **Install ID**: a random UUIDv4, generated on first use, stored in a plain
  file (`telemetry_install_id`) in the app config dir. It is not derived
  from the machine, user, or hardware. Opting out deletes it; opting back in
  (or deleting the file manually) creates a fresh ID unlinkable to the old
  one.
- **Session ID**: a random UUIDv4 generated fresh on every launch and never
  written to disk. It groups events *within* one run of the app — which is
  what makes funnel and drop-off analysis possible — without linking one run
  to the next.
- No other identifier exists. Meetings have local UUIDs, but meeting IDs are
  **not** sent — events are deliberately not joinable to a specific meeting.

### Consent model (settled)

Telemetry is **on by default with prominent disclosure and a one-click
opt-out** in Settings → Privacy. Rationale:

- Everything sent is content-free by construction (see above), which keeps
  the risk profile close to a crash counter, not analytics.
- The stakeholder questions this schema answers (activation funnel, call-
  detection accept rate) are exactly the ones opt-in telemetry cannot
  answer: opt-in rates in desktop apps are low and heavily self-selected,
  which silently biases funnel and reliability data.
- The opt-out is real, and it acts in **four** places: new events stop before
  they reach the queue; the export worker discards anything already queued
  instead of sending it; **the disk spool is deleted**; and the install ID
  file is deleted. Nothing is sent after the toggle goes off, including
  anything the app had saved to disk while it was offline. The spool is also
  purged at startup whenever telemetry is off, so a crash cannot leave a
  stale batch behind for a later session to send.

## 2. Transport

OTLP/HTTP **JSON** logs (`POST <endpoint>/v1/logs`), sent to the Grafana Cloud
OTLP gateway, which forwards to Loki.

> **JSON is confirmed working.** The live gateway was tested with a hand-built
> OTLP/HTTP JSON payload and returned **HTTP 204**. Protobuf is **not**
> required. Please do not reopen this: it is the reason Minutes does not pull
> in the `opentelemetry` / `opentelemetry-otlp` SDK stack (gRPC, `tonic`,
> `prost`, its own `reqwest` feature matrix) for what is a few dozen small
> JSON documents a day.

Each event is one log record with:

- `body` = event name, `event.name` attribute = event name (for LogQL
  filtering)
- Scope: `desksec.telemetry`, version = schema version (currently `1`)
- Resource attributes describing the install (low cardinality — see below)
- Log record attributes carrying the two identifiers and the event's own
  attributes

### 2.1 Resource attributes (low cardinality only)

| Resource attribute | Type | Value | Why it is here |
| --- | --- | --- | --- |
| `service.name` | string | constant `desksec` | `{service_name="desksec"}` is the LogQL selector that isolates this app's data from the other internal tooling sharing the stack (`codex_cli_rs`, `claude-code-desktop`, `cowork`). The name was checked and is free. |
| `service.namespace` | string | constant `amalitech` | Groups Minutes with the org's other services. |
| `service.version` | string | app version, e.g. `0.1.0` | Adoption per release; correlate regressions with a build. |
| `os.type` | string | `macos` \| `windows` \| `linux` | |
| `os.version` | string | coarse **major.minor**, e.g. `15.3`, `22.04`; `10`/`11` on Windows; `unknown` if unavailable | Which OS versions must keep working. **Build strings are deliberately stripped** — they are close to unique in a small population. |
| `device.arch` | string | `aarch64` \| `x86_64` \| … | Apple Silicon vs Intel dominates on-device Whisper speed. Without it, "transcription is slow" is uninterpretable. |
| `device.cpu.cores` | int | logical CPU count, `0` if unavailable | Same reason. Read together with `transcription_latency_bucket`. |
| `app.channel` | string | `debug` \| `release` | Filter developer machines out of adoption numbers. |

`device.*` here means the *class* of machine — CPU architecture and core
count. It never means the name of an audio device, a hostname, or a serial
number; those remain banned by §1.

### 2.2 Cardinality rule

**High-cardinality identifiers go on the log record, never on the resource.**

Resource attributes are the ones an OTLP-to-Loki pipeline is most likely to
promote into Loki **index labels**. A label whose value is unique per user or
per launch is a cardinality blow-up that degrades the whole shared stack, not
just Minutes's data. Nobody can currently confirm the gateway's exact
label-promotion configuration, so this is handled **defensively**: the
resource block above contains only values from small, finite sets.

The two identifiers are therefore log **record** attributes, where they cost
storage rather than cardinality:

| Record attribute | Type | Value | Purpose |
| --- | --- | --- | --- |
| `session.id` | string | random UUIDv4, new every launch, never persisted | Group events inside one run without linking runs. |
| `desksec.install.id` | string | random UUIDv4, resettable, not machine-derived | The only cross-run identifier. |

This is enforced in code (`ALLOWED_RESOURCE_ATTR_KEYS` vs
`IDENTITY_ATTR_KEYS` in `src-tauri/src/telemetry.rs`) and by the test
`resource_attributes_carry_no_high_cardinality_identifiers`, which fails if
either identifier — or any key containing `install` or `session` — ever
appears in the resource block again.

### 2.3 Delivery

Delivery is **asynchronous** end to end: `telemetry::event()` does an atomic
check, an allowlist filter and a `try_send` onto a bounded in-memory queue
(256). Everything else — batching (max 64 per request, flushed at least every
30 s), the 10 s-timeout HTTP request, and every spool file write — happens on
a background task. Telemetry can never block, slow, or error the recording
pipeline, and a full queue drops events rather than waiting.

The install ID is read from disk once per batch rather than cached in memory.
That is what makes "opt out, then opt back in" produce a genuinely fresh ID
in the same session, with no restart and no link to the previous one.

### 2.4 Disk spool and retry

The collector is normally reachable, but a laptop is not always online. A
failed export is therefore persisted and retried instead of dropped.

| Property | Value |
| --- | --- |
| Location | `<app config dir>/telemetry_spool/` |
| File format | one complete OTLP/JSON batch per file, named by timestamp so lexical order is chronological order |
| Partial writes | written as `*.json.tmp` then renamed; a `.tmp` left by a crash is never read back as a batch |
| Max batches | 200 |
| Max total bytes | 5 MB |
| Eviction | **oldest first**, when either cap is exceeded |
| Loss reporting | evicted batches are counted and reported as a `telemetry_spool_dropped` event with a `dropped` count, so a gap in the data is visible rather than silently misleading |
| First retry | ~30 s |
| Backoff | doubles per failed attempt, capped at 30 min |
| Jitter | "equal jitter" — the actual delay is uniform in `[base/2, base]` |
| Startup | the spool is drained before anything else, so events survive an app restart |
| Opt-out | **the whole directory is deleted** |

**What is in those files.** The spool holds the same OTLP payloads that would
have been POSTed: allowlisted metadata only. No transcript, no summary, no
meeting title, no participant name, no file path, no audio. A reader should
not assume "telemetry writes to disk" means meeting data is written to disk —
it is not, and a test (`spooled_payloads_on_disk_contain_no_content`) asserts
it.

**Why jitter.** Without it, every client that was online during an outage
retries on exactly the same schedule and stampedes the collector the moment it
recovers. The lower half of the window is kept fixed so a delay can never
collapse to zero and spin the retry loop.

**Retryable vs not.** Network errors, timeouts, HTTP **429** and **5xx** are
transient: keep the batch and try again later. Every other non-2xx status —
**4xx other than 429**, and unfollowed redirects — means the payload, the URL
or the credentials are wrong. Retrying cannot fix that, so the batch is
**discarded**; a spool that can never drain is just a slow disk leak plus
pointless load on the collector.

**Failure is never fatal.** A full disk, a read-only config dir, a corrupt or
truncated spool file, a file deleted underneath the worker: each of these
costs at most one batch. None of them errors, blocks, or panics.

## 3. Configuration

### 3.1 Destination (hardcoded, non-secret)

Neither of these is a secret, so both are compiled in as defaults and the app
needs no per-machine configuration.

| Constant (`src-tauri/src/telemetry.rs`) | Value |
| --- | --- |
| `DEFAULT_OTLP_ENDPOINT` | `https://otlp-gateway-prod-eu-west-2.grafana.net/otlp` |
| `GRAFANA_INSTANCE_ID` | `1549080` |

The code appends `/v1/logs` to the base, which is the correct signal URL for
this gateway.

### 3.2 The token — `DESKSEC_OTLP_TOKEN`

Grafana Cloud uses HTTP Basic, where the **username is the instance ID** and
the **password is a Grafana Cloud write token**:

```
Authorization: Basic base64("1549080:<token>")
```

This exact shape was verified against the live gateway with the current write
token and returned **HTTP 204**. The header is assembled in code from the
instance ID constant and the token, so the token never has to exist anywhere
in pre-encoded form.

**The token is never stored in this repository** — not in source, not in a
test, not in a fixture, not redacted, not partial. It is intentionally compiled
into the current internal-only, VPN-distributed installer. This is a pragmatic
exception: the shared credential is extractable from the binary and could be
abused for ingestion or billing if an installer leaves the internal trust
boundary. Rotate it if that happens. To configure it:

1. Create a Grafana Cloud OTLP **write token** for stack `1549080`.
2. Add it as a GitHub Actions secret named `DESKSEC_OTLP_TOKEN`. Local
   development can set the same variable in the process environment or `.env`.

The release workflow requires this secret and passes it only to the Tauri build
step. `build.rs` embeds it with `cargo:rustc-env`, and runtime configuration can
still override it for development or staging.

### 3.3 Resolution order

| Rank | Source | Notes |
| --- | --- | --- |
| 1 | `OTEL_EXPORTER_OTLP_HEADERS` / `OTEL_EXPORTER_OTLP_ENDPOINT` (or the Minutes-specific `DESKSEC_TELEMETRY_HEADERS` / `DESKSEC_TELEMETRY_ENDPOINT`) | Explicit headers **replace** the constructed Grafana header entirely, so a developer can point a build at a staging stack or a self-hosted collector without rebuilding, and with no Grafana token at all. |
| 2 | `DESKSEC_OTLP_TOKEN` in the process environment (including `.env`) | Runtime override for local development or staging. |
| 3 | CI-embedded `DESKSEC_OTLP_TOKEN` | Default for internal release builds. Extractable from the binary; accepted only within the current internal/VPN trust boundary. |
| 4 | Nothing | **Completely inert.** No worker, no queue, no spool, no network, silent. |

Minutes never POSTs unauthenticated: without a credential the exporter stays
inert rather than spooling a queue of batches the gateway would reject.

Malformed or placeholder values are treated as unset, for the endpoint and for
the token alike. The endpoint must start with `https://` or `http://`, and it
reuses `settings::is_placeholder_key`, so a value containing `your-`, `-here`,
`placeholder`, `changeme`, `example`, `xxxx`, or `...` is ignored. Note this
means a real host under `example.com` will not be accepted; use the actual
collector hostname.

## 4. Attribute dictionary

All **event** attributes any event may carry (resource attributes are in
[§2](#2-transport)). Types are OTLP types; strings are members of the closed
sets below, never free text.

| Attribute | Type | Values |
| --- | --- | --- |
| `schema_version` | int | `1` |
| `engine` | string | `whisper` \| `deepgram` |
| `whisper_model` | string | `tiny` \| `base` \| `small` \| `medium` \| `large-v3` |
| `diarization` | bool | |
| `capture_microphone` | bool | |
| `capture_system_audio` | bool | |
| `call_detection_enabled` | bool | |
| `export_markdown` | bool | auto-export-to-vault setting |
| `theme` | string | `system` \| `light` \| `dark` |
| `duration_bucket` | string | `<1m`, `1-5m`, `5-15m`, `15-30m`, `30-60m`, `1-2h`, `>2h` |
| `transcript_length_bucket` | string | `0`, `1-1k`, `1k-5k`, `5k-20k`, `20k-50k`, `50k-100k`, `>100k` (characters) |
| `format` | string | `md` \| `txt` \| `docx` |
| `trigger` | string | `manual` \| `call_prompt` |
| `prompt_kind` | string | `manual` \| `call` |
| `error.type` | string | `network` \| `timeout` \| `auth` \| `server` \| `internal` (mirrors `src-tauri/src/error.rs` / `src/utils/errors.ts`) |
| `area` | string | `summary` \| `recording_start` \| `export` \| `transcription_stream` |
| `recovered_count` | int | interrupted meetings recovered at startup |
| `granted` | bool | OS microphone permission outcome |
| `dropped` | int | spooled batches lost to the spool caps ([§2.4](#24-disk-spool-and-retry)) |
| `outcome` | string | `success` \| `failed` \| `cancelled` |
| `app_startup_duration_bucket` | string | latency bucket (below) |
| `summarize_duration_bucket` | string | latency bucket (below) |
| `transcription_latency_bucket` | string | latency bucket (below) |
| `download_duration_bucket` | string | latency bucket (below) |

**Latency buckets** (shared by every `*_duration_bucket` / `*_latency_bucket`
attribute, so they stay comparable): `<0.5s`, `0.5-1s`, `1-3s`, `3-5s`,
`5-10s`, `10-30s`, `30-60s`, `1-3m`, `3-10m`, `>10m`. Raw millisecond timings
are never sent — across many events they are a surprisingly good fingerprint,
and coarse ranges answer every question we actually have.

## 5. Events

Legend: ✅ instrumented in this PR · 🔜 defined here, instrumentation is a
follow-up (the schema is the contract; wiring is one `telemetry::event()`
call).

### 5.1 Lifecycle & activation funnel

| Event | Attributes | Status | Notes |
| --- | --- | --- | --- |
| `app_started` | `schema_version`, `engine`, `diarization`, `capture_microphone`, `capture_system_audio`, `call_detection_enabled` | ✅ | Once per launch. Doubles as the configuration-mix snapshot. |
| `mic_permission_result` | `granted` | 🔜 | Emit after the first capture attempt resolves the OS mic permission. Only granted/denied — nothing else. |
| `recording_started` | `trigger`, `engine`, `diarization`, `capture_microphone`, `capture_system_audio` | ✅ | |
| `recording_start_failed` | `trigger`, `error.type` | ✅ | Start paths return string errors today, so `error.type=internal` until categories reach the recorder. |
| `recording_completed` | `engine`, `duration_bucket` | ✅ | `duration_bucket` × `engine` is the Deepgram streaming-minutes cost proxy. |
| `summary_generated` | `engine`, `transcript_length_bucket`, `summarize_duration_bucket` | ✅ | `transcript_length_bucket` is the summary-model input-token cost proxy. The transcript itself is never sent. |
| `export_completed` | `format` | ✅ | |
| `search_performed` | *(none — count only)* | 🔜 | Emit once per search session (debounced), never per keystroke, and never the query. |

The funnel steps *install → first open → first recording → first summary →
returned in week 2* are **derived in Grafana**, not sent as events: first
occurrence of each event per `desksec.install.id`, and week-2 retention =
installs with any event in days 8–14 after their first event. No client-side
"first_*" flags needed.

### 5.2 Call-detection growth hook

| Event | Attributes | Status | Notes |
| --- | --- | --- | --- |
| `meeting_prompt_shown` | `prompt_kind` | ✅ | `call` = auto-detected, `manual` = New meeting button. |
| `meeting_prompt_accepted` | `prompt_kind` | ✅ | Accept rate = accepted/shown for `prompt_kind=call`. This is the habit-formation signal. |
| `meeting_prompt_dismissed` | `prompt_kind` | ✅ | |

The detected app (Zoom, Teams, …) is **not** reported.

### 5.3 Reliability

| Event | Attributes | Status | Notes |
| --- | --- | --- | --- |
| `error` | `area`, `error.type` | ✅ (summary path) | Category only, never the message. Extend by passing a new `area`. |
| `unclean_shutdown_detected` | `recovered_count` | ✅ | Emitted when startup recovers meetings left in `recording` state — the crash/kill proxy. |
| `telemetry_spool_dropped` | `dropped` | ✅ | Spooled batches evicted by the spool caps ([§2.4](#24-disk-spool-and-retry)). Attached to the next batch that gets through, so a gap in the data is visible instead of silent. |
| `transcription_stream_disconnected` | `engine`, `error.type` | 🔜 | Deepgram websocket drops (`src-tauri/src/remote_stream.rs`). Left out to keep the diff away from the live-streaming path. |

### 5.4 Settings & consent

| Event | Attributes | Status | Notes |
| --- | --- | --- | --- |
| `settings_changed` | `engine`, `diarization`, `capture_system_audio`, `call_detection_enabled`, `theme`, `export_markdown` | 🔜 | Snapshot on save; keeps the configuration mix current between launches. |

Opt-out itself is deliberately **not** an event: once the user says stop,
nothing more is sent — not even "goodbye". Opt-out rates can be inferred
server-side from installs that go silent while the app version stays in
support.

### 5.5 Performance

The stakeholder is explicitly interested in performance. All of these use the
shared latency buckets from [§4](#4-attribute-dictionary), never raw timings.

| Event | Attributes | Status | Notes |
| --- | --- | --- | --- |
| `app_startup_completed` | `app_startup_duration_bucket` | ✅ | Process start → UI ready. "UI ready" is the first `get_settings` call from the webview, which is the first thing `App.tsx` does on mount. Emitted once per launch. |
| `summary_generated` | `summarize_duration_bucket` (with `engine`, `transcript_length_bucket`) | ✅ | Wall clock for the summary request. Read next to `transcript_length_bucket` to see whether slowness tracks transcript size or is slow regardless. |
| `model_download_completed` | `whisper_model`, `outcome`, `download_duration_bucket` | ✅ | A Whisper model download that fails or is abandoned is an activation blocker — the user cannot transcribe anything until it finishes. Never the URL, the path, or the file name. |
| `transcription_chunk_completed` | `engine`, `whisper_model`, `transcription_latency_bucket` | 🔜 | On-device time to transcribe one chunk. **Defined but deliberately not wired.** The only call site is `recorder.rs` → `local_transcribe::transcribe_samples`, which is the live audio path; instrumenting it was not worth any risk to recording. Read together with `device.arch` and `device.cpu.cores`, this is the single best answer to "why is on-device slow for some users", so it is the first thing to wire in a follow-up that can touch that path safely. |

## 6. Retention & access

**Retention: 12 months.** Events older than that are removed. This is the
number the user-facing disclosure in Settings → Privacy is written against, so
changing it means changing that copy too.

Settled:

- **Destination**: Grafana Cloud, stack `1549080`, EU West 2 OTLP gateway,
  forwarded to Loki ([§3.1](#31-destination-hardcoded-non-secret)).
- **Transport**: OTLP/HTTP JSON, confirmed accepted (HTTP 204). Protobuf not
  required ([§2](#2-transport)).
- **Auth**: HTTP Basic, instance ID as username, `DESKSEC_OTLP_TOKEN` as
  password, provisioned as a GitHub Actions secret
  ([§3.2](#32-the-token--desksec_otlp_token)).
- **Consent default**: disclosed-by-default with opt-out
  ([§0](#0-settled-decisions)).
- **Query isolation**: `{service_name="desksec"}` selects Minutes's data only.
  The stack is shared with `codex_cli_rs`, `claude-code-desktop` and `cowork`;
  the name `desksec` was checked and is free.

Still open:

1. Whether a **staging tenant** exists for testing before release builds point
   at production.

**Internal release builds are credentialed by CI.** Builds made without either
an embedded or runtime `DESKSEC_OTLP_TOKEN` remain inert: no worker, queue,
spool, or network traffic.

## 7. Adding a new event (checklist)

1. Re-read section 1. If the thing you want to send is not a count, bucket,
   flag, or member of a closed category set — stop.
2. Add any new attribute key to `ALLOWED_ATTR_KEYS` in
   `src-tauri/src/telemetry.rs` (the allowlist test will force you to keep
   it content-free) and to the dictionary in section 4. A new *resource*
   attribute additionally goes in `ALLOWED_RESOURCE_ATTR_KEYS`, in
   `build_export_payload`, and in the table in section 2 — a test checks all
   of them agree. Before adding a resource attribute, re-read
   [§2.2](#22-cardinality-rule): if its value is not from a small, finite set,
   it belongs on the log record, not the resource.
3. Add the event to section 5 with its exact attributes.
4. Call `telemetry::event("your_event", &[...])` at the call site. It is
   asynchronous and safe everywhere, including before init and when the user
   has opted out.
5. For anything timed, use `telemetry::latency_bucket_ms()` — never a raw
   duration.
6. Bump `SCHEMA_VERSION` if you changed the *meaning* of anything existing.
