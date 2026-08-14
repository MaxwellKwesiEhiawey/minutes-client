import { describe, it, expect } from "vitest";
import {
  formatDuration,
  meetingDurationMs,
  formatStatusLabel,
  statusBadgeClass,
  sanitizeFilename,
} from "./format";

describe("formatDuration", () => {
  it("formats hours:minutes:seconds zero-padded", () => {
    expect(formatDuration(0)).toBe("00:00:00");
    expect(formatDuration(1000)).toBe("00:00:01");
    expect(formatDuration(61_000)).toBe("00:01:01");
    expect(formatDuration(3_661_000)).toBe("01:01:01");
  });

  it("clamps negative input to zero", () => {
    expect(formatDuration(-5000)).toBe("00:00:00");
  });
});

describe("meetingDurationMs", () => {
  it("computes elapsed between start and end", () => {
    expect(
      meetingDurationMs("2026-01-01T00:00:00Z", "2026-01-01T01:00:00Z"),
    ).toBe(3_600_000);
  });

  it("never returns negative for out-of-order timestamps", () => {
    expect(
      meetingDurationMs("2026-01-01T01:00:00Z", "2026-01-01T00:00:00Z"),
    ).toBe(0);
  });
});

describe("status helpers", () => {
  it("labels a recording meeting regardless of stored status", () => {
    expect(formatStatusLabel("completed", true)).toBe("Recording");
    expect(formatStatusLabel("interrupted", false)).toBe("Interrupted");
    expect(formatStatusLabel("completed", false)).toBe("Completed");
  });

  it("maps status to a badge class", () => {
    expect(statusBadgeClass("completed", true)).toBe("badge badge-rec");
    expect(statusBadgeClass("interrupted", false)).toBe("badge badge-warn");
    expect(statusBadgeClass("completed", false)).toBe("badge badge-done");
  });
});

describe("sanitizeFilename", () => {
  it("keeps a simple, already-safe title as-is", () => {
    expect(sanitizeFilename("Weekly-Sync_2026")).toBe("Weekly-Sync_2026");
  });

  it("replaces unsafe characters (path separators, colons, etc.) with underscores", () => {
    expect(sanitizeFilename("Q3: Budget / Review")).toBe("Q3_Budget_Review");
    expect(sanitizeFilename("../../etc/passwd")).toBe("_etc_passwd");
  });

  it("truncates to 60 characters", () => {
    const long = "a".repeat(200);
    expect(sanitizeFilename(long)).toHaveLength(60);
  });

  it("falls back to the default name when the title sanitizes to empty", () => {
    expect(sanitizeFilename("🎉🎉🎉")).toBe("meeting");
    expect(sanitizeFilename("")).toBe("meeting");
  });

  it("accepts a custom fallback", () => {
    expect(sanitizeFilename("", "export")).toBe("export");
  });
});
