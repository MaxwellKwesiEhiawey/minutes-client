import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { en } from "./en";

/**
 * The backend and the dictionaries are two halves of one contract: Rust sends
 * `CategorizedError::coded("error.x", …)` and the UI looks `error.x` up. Nothing
 * in the type system connects them, so this reads the Rust sources and checks
 * every code it emits has a string here.
 *
 * A missing key is not a crash — `errorText` falls back to the backend's English
 * — which is exactly why it needs a test: it would ship silently.
 */
const RUST_DIR = join(import.meta.dirname, "../../src-tauri/src");

function rustSources(): string[] {
  return readdirSync(RUST_DIR)
    .filter((f) => f.endsWith(".rs"))
    .map((f) => readFileSync(join(RUST_DIR, f), "utf8"));
}

function emittedCodes(): string[] {
  const codes = new Set<string>();
  for (const source of rustSources()) {
    for (const m of source.matchAll(/coded(?:_with)?\(\s*"([^"]+)"/g)) {
      codes.add(m[1]);
    }
    // Event payloads carry the same contract by hand, e.g. the live-stream
    // warning emits json!({ "code": "error.x", … }). Those were invisible to
    // the pattern above, so a missing translation there shipped silently —
    // precisely the failure this file exists to prevent.
    for (const m of source.matchAll(/"code":\s*"(error\.[^"]+)"/g)) {
      codes.add(m[1]);
    }
  }
  return [...codes].sort();
}

describe("backend error codes", () => {
  it("finds the codes the backend emits", () => {
    // Guards the regex itself: if this ever reads zero, the assertions below
    // would pass by vacuously checking nothing.
    expect(emittedCodes().length).toBeGreaterThan(5);
  });

  it("has a translation for every code the backend emits", () => {
    const missing = emittedCodes().filter((code) => !(code in en));
    expect(missing).toEqual([]);
  });

  it("uses the error. prefix for all of them", () => {
    const odd = emittedCodes().filter((code) => !code.startsWith("error."));
    expect(odd).toEqual([]);
  });
});
