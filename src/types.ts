export type PositionMode =
  | "random"
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-right"
  | "fixed";

export type MonitorMode = "cursor" | "primary" | "random";

export type TriggerModifier = "primary" | "control" | "alt" | "shift" | "meta";

type Letter =
  | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
  | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z";
type Digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
type FunctionNumber =
  | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12"
  | "13" | "14" | "15" | "16" | "17" | "18" | "19" | "20" | "21" | "22" | "23" | "24";

export type CommandKey =
  | `Key${Letter}`
  | `Digit${Digit}`
  | `F${FunctionNumber}`
  | `Numpad${Digit}`
  | "ArrowUp"
  | "ArrowDown"
  | "ArrowLeft"
  | "ArrowRight"
  | "Home"
  | "End"
  | "PageUp"
  | "PageDown"
  | "Insert"
  | "Delete"
  | "Escape"
  | "Tab"
  | "Space"
  | "Enter"
  | "Backspace"
  | "Backquote"
  | "Minus"
  | "Equal"
  | "BracketLeft"
  | "BracketRight"
  | "Backslash"
  | "Semicolon"
  | "Quote"
  | "Comma"
  | "Period"
  | "Slash"
  | "NumpadAdd"
  | "NumpadSubtract"
  | "NumpadMultiply"
  | "NumpadDivide"
  | "NumpadEnter"
  | "NumpadDecimal"
  | "NumLock"
  | "PrintScreen"
  | "ScrollLock"
  | "Pause"
  | "ContextMenu"
  | "IntlBackslash";

export interface TriggerChord {
  modifiers: TriggerModifier[];
  key: CommandKey;
}

export interface TriggerCommand {
  version: 1;
  steps: TriggerChord[];
}

export interface AppConfig {
  locale: "zh-CN" | "en";
  enabled: boolean;
  triggerCommand: TriggerCommand;
  cooldownMs: number;
  sequenceTimeoutMs: number;
  loops: number;
  scale: number;
  position: {
    mode: PositionMode;
    monitor: MonitorMode;
    margin: number;
    fixedX: number;
    fixedY: number;
  };
  petManifestPath: string | null;
}

export type AnimationName =
  | "idle"
  | "running-right"
  | "running-left"
  | "waving"
  | "jumping"
  | "failed"
  | "web-swing"
  | "spider-upside-down"
  | "spider-heart"
  | "spider-crying"
  | "spider-showcase"
  | "waiting"
  | "running"
  | "review";

export interface PetChoice {
  manifestPath: string;
  id: string;
  displayName: string;
  description: string;
}

export interface PetThumbnail {
  id: string;
  displayName: string;
  description: string;
  imageDataUrl: string;
}

export interface MarketPet {
  rank: number;
  id: string;
  displayName: string;
  description: string;
  creator: string;
  likeCount: number;
  posterUrl: string;
  previewUrl: string;
  tags: string[];
  installed: boolean;
  manifestPath: string | null;
}

export interface MarketCatalog {
  pets: MarketPet[];
  source: "live" | "cache" | "bundled";
  warning: string | null;
}

export interface MarketInstallResult {
  id: string;
  manifestPath: string;
  alreadyInstalled: boolean;
}

export interface MarketInstallSummary {
  installed: number;
  skipped: number;
  failed: number;
}

export interface MarketInstallProgress {
  current: number;
  total: number;
  id: string;
  displayName: string;
  status: "installed" | "skipped" | "failed";
  error: string | null;
}

export interface PetPayload {
  manifest: {
    id: string;
    displayName: string;
    description: string;
    spritesheetPath: string;
  };
  spritesheetDataUrl: string;
  frameWidth: number;
  frameHeight: number;
  columns: number;
  rows: number;
}

export interface RuntimeStatus {
  paused: boolean;
  listenerRunning: boolean;
  listenerError: string | null;
  platform: string;
  inputScope: string;
  triggerCount: number;
  overlayRenderCount: number;
  overlayReady: boolean;
}
