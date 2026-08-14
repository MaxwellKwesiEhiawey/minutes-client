# Contributing to Minutes

## The vendored `minutes/` dependency

`src-tauri/Cargo.toml` depends on `minutes-core` via a **path dependency**:

```toml
minutes-core = { path = "../minutes/crates/core", default-features = false, features = ["whisper", "diarize"] }
```

`minutes/` is a vendored copy of the `silverstein/minutes` monorepo, checked
into this repo (not a git submodule, not a crates.io dependency). It provides
the on-device transcription (whisper.cpp) and speaker diarization
(pyannote-rs) engine that Minutes's Rust backend calls into. Because it's a
path dependency, Cargo resolves `minutes-core`'s own dependencies **into the
same dependency graph** as Minutes's — they are not isolated from each other.

That has one important consequence: a handful of dependencies in
`src-tauri/Cargo.toml` are pinned to match the exact (or closely exact)
version `minutes-core` itself uses, specifically:

- **`ort = "=2.0.0-rc.10"`** — `pyannote-rs 0.3.4` (pulled in by
  `minutes-core`'s diarization feature) does not compile against newer `ort`
  release candidates; they made `OrtCustomOpDomain` `!Send`/`!Sync`. Since
  Minutes's own `Cargo.toml` also declares an `ort` dependency (outside the
  Minutes workspace), Cargo would otherwise be free to pick a newer,
  incompatible RC. The `=` pin removes that freedom.
- **`rusqlite = { version = "0.33", features = ["bundled"] }`** — pinned to
  match the version `minutes-core` uses. `rusqlite`'s underlying
  `libsqlite3-sys` crate declares `links = "sqlite3"`, and Cargo's `links` key
  means only **one** version of a crate with that key can be linked into a
  single binary. If Minutes and `minutes-core` resolved to two different
  `rusqlite`/`libsqlite3-sys` versions, the build would fail outright with a
  "multiple packages link to native library" error, not just silently
  duplicate the dependency the way `reqwest` did (see below).
- **`cpal = "0.18.1"`** — same reasoning as `rusqlite`: `cpal`'s backend on
  Linux links the native `alsa` library, so two different `cpal` versions
  resolving in the same binary risks a similar native-link conflict.

**These pins are load-bearing, not stylistic.** Bumping any of the three in
isolation, without checking what `minutes/crates/core/Cargo.toml` (and its
own `Cargo.lock`, if vendored with one) declares, risks a build break that
manifests as an obscure linker error rather than a normal Rust type error.

### Re-vendoring `minutes/`

If you pull in a newer copy of the `silverstein/minutes` monorepo:

1. Diff `minutes/crates/core/Cargo.toml` (and any other vendored crate under
   `minutes/crates/*` that Minutes's dependency tree reaches) against the
   previous version, specifically looking at `ort`, `rusqlite`, and `cpal`.
2. Update the matching pins in `src-tauri/Cargo.toml` to stay aligned.
3. Run `cargo update -p ort -p rusqlite -p cpal` in `src-tauri/` and inspect
   what `Cargo.lock` actually resolves — Cargo will error loudly on the
   `ort` exact-version conflict if the pins are out of sync, but the
   `links = "sqlite3"` / native `alsa` conflicts for `rusqlite`/`cpal` can be
   quieter (a successful resolve that still fails at link time).
4. Run `cargo check && cargo test && cargo clippy --all-targets -- -D warnings`
   in `src-tauri/` before opening a PR. (See "Verifying Rust changes" below —
   this project's CI does this for you on every PR via
   `.github/workflows/build-desktop.yml`'s `rust-checks` job, but it's much
   faster to catch locally first.)

### The `reqwest` duplication (already fixed, kept here as a cautionary example)

A previous version of this file's reasoning attributed a duplicate
`reqwest 0.12`/`0.13` resolution to `minutes-core`. That was wrong — the
actual cause was that `tauri` 2.11.x itself depends on `reqwest 0.13.4`,
unrelated to the vendored crate. Minutes's own `reqwest` dependency has since
been bumped to the `0.13` line to match (see the comment on the `reqwest`
line in `src-tauri/Cargo.toml`). The lesson: when you see two versions of the
same crate in `Cargo.lock`, check **all** of the dependency graph's roots
(including Tauri and its plugins), not just the vendored path dependency —
it's an easy, plausible-sounding wrong guess to make.

## Verifying Rust changes

Some changes to `src-tauri/` in this repo's history were made in an
environment without a Rust toolchain available (no `cargo`/`rustc`, no
network access to install one) and are explicitly flagged in code comments
as "NOT verified with a local `cargo build`". If you see that phrase in a
comment near code you're touching, treat it as unverified until you've
personally run:

```sh
cd src-tauri
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All four of these also run in CI (`.github/workflows/build-desktop.yml`,
`rust-checks` job) and are blocking — a failure there will block merge — but
it's faster to find out locally first.

## Running the CI gates locally

Before pushing, `npm run ci` from the repo root reproduces both blocking gate
jobs in `.github/workflows/build-desktop.yml` — `frontend-checks` (typecheck,
lint, unit tests, `npm audit --omit=dev --audit-level=high`) followed by
`rust-checks` (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`). It stops at the first failure.

Two things it deliberately leaves out:

- **`cargo audit`** — CI installs the tool on the fly and treats an install
  failure as a CI-infra issue rather than a code finding. Run it yourself with
  `cargo install cargo-audit --locked` once, then `cargo audit` from `src-tauri/`.
- **The `build` matrix** — the four platform legs bundle native code. Locally you
  can only reproduce the macOS legs, e.g.
  `npm run tauri build -- --target aarch64-apple-darwin`. Linux and Windows have
  to be validated by CI.

## Cutting a public release

Minutes ships free public installers via **GitHub Releases** (not OneDrive).
The `release` job in `.github/workflows/build-desktop.yml` runs only when you
push a tag matching `v*` (for example `v0.1.0`).

### Will you get every platform?

Yes — if every matrix leg succeeds. The build matrix produces:

| Artifact name | Platform |
| --- | --- |
| `desksec-macos-aarch64` | macOS Apple Silicon (`.dmg`) |
| `desksec-macos-x86_64` | macOS Intel (`.dmg`) |
| `desksec-linux` | Linux (`.AppImage` / `.deb` / `.rpm`) |
| `desksec-windows` | Windows (`.msi` / `.exe`) |

`fail-fast: false` lets siblings finish if one leg flakes, but the `release`
job still `needs: build`, so a single red platform **blocks** publishing. That
is intentional: public tags should not ship missing platforms. Re-run the
failed job (or fix and retag) before users see a Release.

### Steps

1. Land the commit on `main` (or whatever branch you tag from). The tagged
   commit **must** include `.github/workflows/build-desktop.yml`.
2. Align `src-tauri/tauri.conf.json` `version` with the tag (e.g. `0.1.0` ↔
   `v0.1.0`).
3. Confirm repo **Actions → Variables** has `DESKSEC_API_URL` and **Actions →
   Secrets** has `DESKSEC_TOKEN` and `DESKSEC_OTLP_TOKEN`. These are baked into
   internal release builds at compile time; the server token is seeded into the
   OS keychain on first launch. Release tag builds fail if any are missing.
4. For Gatekeeper-friendly macOS installs, set Apple signing/notarization
   secrets (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`,
   `APPLE_PASSWORD`, `APPLE_TEAM_ID`). Without them CI falls back to ad-hoc
   signing and users may need **Right-click → Open**.
5. Tag and push:

   ```bash
   git checkout main
   git pull
   git tag -a v0.1.0 -m "Minutes v0.1.0"
   git push origin v0.1.0
   ```

6. Watch **Actions → Build Minutes Desktop App** for the tag. When `release`
   finishes, open **Releases** on GitHub — the tag is published immediately
   (`draft: false`) with flattened installer assets.
7. Share `https://github.com/<org>/<repo>/releases/tag/v0.1.0` (or the
   “Latest” release URL).

### Optional: update an existing tag’s release

If you need to replace assets, delete the GitHub Release (and optionally the
tag), fix the build, then retag — or bump to `v0.1.1`. Avoid force-moving tags
that people may already have bookmarked.
