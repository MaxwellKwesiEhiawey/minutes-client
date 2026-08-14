/// Bake a build-environment variable into the binary as a compile-time
/// constant, readable with `option_env!(rustc_key)`.
///
/// `legacy` is a Parley-era fallback name and is optional: pass `None` for
/// variables introduced after the rename. A missing or empty value is not an
/// error — it simply means the feature that reads it stays switched off.
fn embed_env(primary: &str, legacy: Option<&str>, rustc_key: &str) {
    println!("cargo:rerun-if-env-changed={primary}");
    if let Some(legacy) = legacy {
        println!("cargo:rerun-if-env-changed={legacy}");
    }
    let value = std::env::var(primary)
        .or_else(|_| std::env::var(legacy.unwrap_or(primary)))
        .unwrap_or_default();
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    // Baked into the release binary at compile time (set in CI during `tauri build`).
    println!("cargo:rustc-env={rustc_key}={value}");
}

/// Compile the CoreAudio mic-activity helper and stage it for bundling.
///
/// Without this, `call_detect::find_mic_check_binary` finds nothing in a
/// packaged build and `run_mic_check_script` falls back to writing the embedded
/// Swift to a temp file and running `swift` on it — which needs Xcode Command
/// Line Tools on the *user's* machine. Where those are missing the probe returns
/// `None`, `is_mic_in_use()` is always false, and call detection silently never
/// fires. Onboarding tells people detection is set up, so that has to be true.
///
/// A missing `swiftc` is a warning, not a build failure: a contributor without
/// Xcode should still be able to build the app, and the runtime `swift` fallback
/// still covers them locally.
#[cfg(target_os = "macos")]
fn compile_mic_check_helper() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest_dir.join("src/mic_check.swift");
    println!("cargo:rerun-if-changed={}", source.display());

    let target = std::env::var("TARGET").unwrap_or_default();
    let bin_dir = manifest_dir.join("bin");
    if let Err(e) = std::fs::create_dir_all(&bin_dir) {
        println!("cargo:warning=could not create {}: {e}", bin_dir.display());
        return;
    }

    // Tauri's `externalBin` resolves `bin/mic_check` to `bin/mic_check-<triple>`
    // and drops it next to the executable in the bundle, which is the first place
    // `find_mic_check_binary` looks.
    let triple_path = bin_dir.join(format!("mic_check-{target}"));
    let status = std::process::Command::new("swiftc")
        .arg("-O")
        .arg(&source)
        .arg("-o")
        .arg(&triple_path)
        .status();

    match status {
        Ok(status) if status.success() => {
            // Second copy under the plain name for the dev path, which looks in
            // `$CARGO_MANIFEST_DIR/bin/mic_check` when running unbundled.
            let plain = bin_dir.join("mic_check");
            if let Err(e) = std::fs::copy(&triple_path, &plain) {
                println!("cargo:warning=could not stage {}: {e}", plain.display());
            }
        }
        Ok(status) => println!(
            "cargo:warning=swiftc failed ({status}) building mic_check; call detection will fall back to a runtime `swift` compile"
        ),
        Err(e) => println!(
            "cargo:warning=swiftc unavailable ({e}); call detection will fall back to a runtime `swift` compile, which needs Xcode Command Line Tools on the user's machine"
        ),
    }
}

fn main() {
    // Server URL and bearer token are baked in at CI build time for zero-config
    // internal installs. On first launch the token is copied into the OS
    // credential store; embedded values win over local configuration.
    embed_env(
        "DESKSEC_API_URL",
        Some("PARLEY_API_URL"),
        "DESKSEC_EMBEDDED_API_URL",
    );
    embed_env(
        "DESKSEC_TOKEN",
        Some("PARLEY_TOKEN"),
        "DESKSEC_EMBEDDED_TOKEN",
    );

    // Pragmatic exception for the internal-only, VPN-distributed desktop app:
    // CI embeds the shared Grafana write token so employee installs have live
    // telemetry without runtime provisioning. This token is still extractable
    // from the binary and must be rotated if an installer leaves the internal
    // trust boundary.
    embed_env("DESKSEC_OTLP_TOKEN", None, "DESKSEC_OTLP_TOKEN");

    #[cfg(target_os = "macos")]
    compile_mic_check_helper();

    tauri_build::build();
}
