import { describe, expect, it } from "vitest";
import { shortModelName } from "./modelName";

describe("shortModelName", () => {
  it("returns the last path segment of a provider model path", () => {
    expect(shortModelName("accounts/fireworks/models/gpt-oss-120b")).toBe(
      "gpt-oss-120b",
    );
  });

  it("returns a plain model id unchanged", () => {
    expect(shortModelName("claude-sonnet-4-5")).toBe("claude-sonnet-4-5");
  });

  it("ignores a trailing slash", () => {
    expect(shortModelName("models/foo/")).toBe("foo");
  });

  it("falls back to the input for degenerate strings", () => {
    expect(shortModelName("")).toBe("");
    expect(shortModelName("/")).toBe("/");
  });
});
