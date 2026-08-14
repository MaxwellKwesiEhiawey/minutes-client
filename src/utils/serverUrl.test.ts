import { describe, expect, it } from "vitest";
import { serverUrlProblem } from "./serverUrl";
import { translatorFor } from "../i18n";

// The real English translator, so this also covers the messages themselves.
const t = translatorFor("en");
const problem = (url: string) => serverUrlProblem(url, t);

// Mirrors the Rust backend's `validate_server_url` (src-tauri/src/settings.rs)
// so a client-side hint can be shown before save, not just after. This test
// exists specifically to keep the two in sync — if the backend's allowed
// localhost variants change, this should be updated to match, and vice versa.
describe("serverUrlProblem", () => {
  it("allows an empty value (means 'leave unchanged' on save)", () => {
    expect(problem("")).toBeNull();
    expect(problem("   ")).toBeNull();
  });

  it("allows any https:// URL", () => {
    expect(problem("https://desksec.example.com")).toBeNull();
    expect(problem("https://192.168.1.5:8787")).toBeNull();
  });

  it("allows http:// only for localhost and loopback variants", () => {
    expect(problem("http://localhost:8787")).toBeNull();
    expect(problem("http://127.0.0.1:8787")).toBeNull();
    expect(problem("http://sub.localhost:8787")).toBeNull();
    expect(problem("HTTP://LOCALHOST:8787")).toBeNull();
  });

  it("rejects http:// for a non-local host — this is the security-relevant case", () => {
    const err = problem("http://desksec.example.com");
    expect(err).not.toBeNull();
    expect(err).toMatch(/https/i);
  });

  it("rejects a hostname that merely contains 'localhost' as a substring, not a real localhost host", () => {
    // e.g. http://localhost.evil.com must NOT be treated as loopback.
    const err = problem("http://localhost.evil.com");
    expect(err).not.toBeNull();
  });

  it("rejects unparseable input", () => {
    expect(problem("not a url")).not.toBeNull();
  });

  it("rejects non-http(s) schemes", () => {
    expect(problem("ftp://example.com")).not.toBeNull();
    expect(problem("file:///etc/passwd")).not.toBeNull();
  });
});
