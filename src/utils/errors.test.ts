import { describe, expect, it } from "vitest";
import { errorMessage, normalizeError } from "./errors";

describe("normalizeError", () => {
  it("passes through a well-formed structured error unchanged", () => {
    const e = { kind: "network", message: "Could not reach the server." };
    expect(normalizeError(e)).toEqual(e);
  });

  it("ignores an unrecognized kind and treats it as an unstructured value", () => {
    const e = { kind: "bogus", message: "nope" };
    expect(normalizeError(e).kind).toBe("internal");
  });

  it("unwraps a native Error's message", () => {
    expect(normalizeError(new Error("boom"))).toEqual({
      kind: "internal",
      message: "boom",
    });
  });

  it("treats a plain string rejection as internal (the legacy CmdResult<T,String> shape)", () => {
    expect(normalizeError("meeting not found")).toEqual({
      kind: "internal",
      message: "meeting not found",
    });
  });

  it("never throws and never produces the '[object Object]' footgun for a random object", () => {
    const weird = { foo: "bar" };
    const result = normalizeError(weird);
    expect(result.kind).toBe("internal");
    expect(result.message).not.toBe("[object Object]");
    expect(result.message).toContain("foo");
  });

  it("handles null/undefined without throwing", () => {
    expect(normalizeError(null).message).toBeTruthy();
    expect(normalizeError(undefined).message).toBeTruthy();
  });

  it("errorMessage is a shorthand for normalizeError(e).message", () => {
    expect(errorMessage("plain string")).toBe("plain string");
  });
});
