import { describe, expect, it } from "vitest";
import {
  chordForKeyboardEvent,
  commandKeyForCode,
  formatTriggerCommand,
  modifiersForKeyboardEvent,
} from "./trigger-command";

const noModifiers = { altKey: false, ctrlKey: false, metaKey: false, shiftKey: false };

describe("trigger command recording", () => {
  it("records the physical key code instead of localized text", () => {
    expect(chordForKeyboardEvent({ code: "KeyC", ...noModifiers }, "macos")).toEqual({
      modifiers: [],
      key: "KeyC",
    });
    expect(commandKeyForCode("KeyÜ")).toBeNull();
  });

  it("maps each platform's conventional shortcut modifier to primary", () => {
    expect(modifiersForKeyboardEvent({ ...noModifiers, metaKey: true }, "macos"))
      .toEqual(["primary"]);
    expect(modifiersForKeyboardEvent({ ...noModifiers, ctrlKey: true }, "windows"))
      .toEqual(["primary"]);
    expect(modifiersForKeyboardEvent({ ...noModifiers, ctrlKey: true }, "macos"))
      .toEqual(["control"]);
    expect(modifiersForKeyboardEvent({ ...noModifiers, metaKey: true }, "windows"))
      .toEqual(["meta"]);
  });

  it("captures a shortcut as one chord and repeated space as two steps", () => {
    const copy = chordForKeyboardEvent(
      { code: "KeyC", ...noModifiers, metaKey: true },
      "macos",
    );
    expect(copy).toEqual({ modifiers: ["primary"], key: "KeyC" });
    expect(formatTriggerCommand({ version: 1, steps: [copy!] }, "macos")).toBe("⌘ + C");
    expect(formatTriggerCommand({
      version: 1,
      steps: [
        { modifiers: [], key: "Space" },
        { modifiers: [], key: "Space" },
      ],
    }, "windows")).toBe("Space → Space");
  });

  it("folds Windows numpad Enter into Enter because the native hook cannot distinguish it", () => {
    expect(chordForKeyboardEvent({ code: "NumpadEnter", ...noModifiers }, "windows"))
      .toEqual({ modifiers: [], key: "Enter" });
    expect(chordForKeyboardEvent({ code: "NumpadEnter", ...noModifiers }, "macos"))
      .toEqual({ modifiers: [], key: "NumpadEnter" });
  });

  it("ignores modifier-only, auto-repeat, composing, and unsupported keys", () => {
    expect(chordForKeyboardEvent({ code: "MetaLeft", ...noModifiers }, "macos")).toBeNull();
    expect(chordForKeyboardEvent({ code: "Space", ...noModifiers, repeat: true }, "macos"))
      .toBeNull();
    expect(chordForKeyboardEvent({ code: "KeyA", ...noModifiers, isComposing: true }, "macos"))
      .toBeNull();
    expect(chordForKeyboardEvent({ code: "AudioVolumeUp", ...noModifiers }, "macos"))
      .toBeNull();
  });
});
