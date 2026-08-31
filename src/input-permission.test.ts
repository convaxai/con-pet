import { describe, expect, it } from "vitest";
import { shouldAutoRequestInputMonitoring } from "./input-permission";

describe("input monitoring permission bootstrap", () => {
  it("automatically requests missing macOS permission once", () => {
    expect(shouldAutoRequestInputMonitoring("macos", false, false)).toBe(true);
    expect(shouldAutoRequestInputMonitoring("macos", false, true)).toBe(false);
  });

  it("does not request on other platforms or when permission is granted", () => {
    expect(shouldAutoRequestInputMonitoring("windows", false, false)).toBe(false);
    expect(shouldAutoRequestInputMonitoring("macos", true, false)).toBe(false);
  });
});
