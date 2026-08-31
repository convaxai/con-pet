export type PositionMode =
  | "random"
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-right"
  | "fixed";

export type MonitorMode = "cursor" | "primary" | "random";

export interface AppConfig {
  locale: "zh-CN" | "en";
  enabled: boolean;
  keyword: string;
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
