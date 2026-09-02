import type {
  CommandKey,
  TriggerChord,
  TriggerCommand,
  TriggerModifier,
} from "./types";

export type DesktopPlatform = "macos" | "windows" | "linux" | string;

export const MAX_TRIGGER_STEPS = 64;

const MODIFIER_CODES = new Set([
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "MetaLeft",
  "MetaRight",
]);

const NAMED_KEYS = new Set<CommandKey>([
  "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
  "Home", "End", "PageUp", "PageDown", "Insert", "Delete",
  "Escape", "Tab", "Space", "Enter", "Backspace",
  "Backquote", "Minus", "Equal", "BracketLeft", "BracketRight",
  "Backslash", "Semicolon", "Quote", "Comma", "Period", "Slash",
  "NumpadAdd", "NumpadSubtract", "NumpadMultiply", "NumpadDivide",
  "NumpadEnter", "NumpadDecimal", "NumLock", "PrintScreen", "ScrollLock",
  "Pause", "ContextMenu", "IntlBackslash",
]);

const KEY_LABELS: Partial<Record<CommandKey, string>> = {
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  PageUp: "Page Up",
  PageDown: "Page Down",
  Backspace: "Backspace",
  Delete: "Delete",
  Escape: "Esc",
  Tab: "Tab",
  Backquote: "`",
  Minus: "−",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  NumpadAdd: "Num +",
  NumpadSubtract: "Num −",
  NumpadMultiply: "Num ×",
  NumpadDivide: "Num ÷",
  NumpadEnter: "Num Enter",
  NumpadDecimal: "Num .",
  NumLock: "Num Lock",
  PrintScreen: "Print Screen",
  ScrollLock: "Scroll Lock",
  ContextMenu: "Menu",
  IntlBackslash: "Intl \\",
};

export interface KeyboardCommandEvent {
  code: string;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  repeat?: boolean;
  isComposing?: boolean;
}

export function isModifierCode(code: string): boolean {
  return MODIFIER_CODES.has(code);
}

export function commandKeyForCode(code: string): CommandKey | null {
  if (/^Key[A-Z]$/.test(code)) return code as CommandKey;
  if (/^Digit[0-9]$/.test(code)) return code as CommandKey;
  if (/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(code)) return code as CommandKey;
  if (/^Numpad[0-9]$/.test(code)) return code as CommandKey;
  return NAMED_KEYS.has(code as CommandKey) ? code as CommandKey : null;
}

export function modifiersForKeyboardEvent(
  event: Pick<KeyboardCommandEvent, "altKey" | "ctrlKey" | "metaKey" | "shiftKey">,
  platform: DesktopPlatform,
): TriggerModifier[] {
  const modifiers: TriggerModifier[] = [];
  if (platform === "macos") {
    if (event.metaKey) modifiers.push("primary");
    if (event.ctrlKey) modifiers.push("control");
  } else {
    if (event.ctrlKey) modifiers.push("primary");
    if (event.metaKey) modifiers.push("meta");
  }
  if (event.altKey) modifiers.push("alt");
  if (event.shiftKey) modifiers.push("shift");
  return modifiers;
}

export function chordForKeyboardEvent(
  event: KeyboardCommandEvent,
  platform: DesktopPlatform,
): TriggerChord | null {
  if (event.repeat || event.isComposing || isModifierCode(event.code)) return null;
  let key = commandKeyForCode(event.code);
  if (!key) return null;
  // keytap currently reports both Return keys as Enter on Windows, so the
  // recorder must not persist an unmatchable NumpadEnter distinction there.
  if (platform === "windows" && key === "NumpadEnter") key = "Enter";
  return { modifiers: modifiersForKeyboardEvent(event, platform), key };
}

export function modifierLabel(modifier: TriggerModifier, platform: DesktopPlatform): string {
  if (modifier === "primary") return platform === "macos" ? "⌘" : "Ctrl";
  if (modifier === "control") return platform === "macos" ? "⌃" : "Control";
  if (modifier === "alt") return platform === "macos" ? "⌥" : "Alt";
  if (modifier === "shift") return platform === "macos" ? "⇧" : "Shift";
  return platform === "windows" ? "Win" : "Meta";
}

export function commandKeyLabel(key: CommandKey, spaceLabel = "Space", enterLabel = "Enter"): string {
  if (key === "Space") return spaceLabel;
  if (key === "Enter") return enterLabel;
  if (key.startsWith("Key")) return key.slice(3);
  if (key.startsWith("Digit")) return key.slice(5);
  if (/^F\d+$/.test(key)) return key;
  if (/^Numpad\d$/.test(key)) return `Num ${key.slice(-1)}`;
  return KEY_LABELS[key] ?? key;
}

export function formatTriggerCommand(
  command: TriggerCommand,
  platform: DesktopPlatform,
  spaceLabel = "Space",
  enterLabel = "Enter",
): string {
  return command.steps
    .map((step) => [
      ...step.modifiers.map((modifier) => modifierLabel(modifier, platform)),
      commandKeyLabel(step.key, spaceLabel, enterLabel),
    ].join(" + "))
    .join(" → ");
}
