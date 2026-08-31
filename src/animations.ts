import type { AnimationName } from "./types";

export interface AnimationDefinition {
  label: string;
  frames: readonly SpriteFrame[];
}

export interface SpriteFrame {
  column: number;
  row: number;
  duration: number;
  offsetX?: number;
  offsetY?: number;
  rotation?: number;
}

const rowFrames = (row: number, durations: readonly number[]): SpriteFrame[] =>
  durations.map((duration, column) => ({ column, row, duration }));

const spiderManSwingOrder = [
  // Atlas cell 0 contains only a few stray web pixels and is visually empty.
  // Start at the first complete pose so the pet never flashes transparent.
  ...Array.from({ length: 19 }, (_, index) => index + 1),
  ...Array.from({ length: 18 }, (_, index) => 18 - index),
  ...Array.from({ length: 18 }, (_, index) => index + 2),
];

const spiderManSwingFrames: SpriteFrame[] = spiderManSwingOrder.map((frameIndex) => {
  const progress = (frameIndex - 1) / 18;
  const atTurningPoint = frameIndex === 1 || frameIndex === 19;
  return {
    column: frameIndex % 8,
    row: Math.floor(frameIndex / 8),
    duration: atTurningPoint ? 180 : 90,
    offsetX: Math.round((progress - 0.5) * 124),
    offsetY: Math.round(Math.sin(progress * Math.PI) * 18),
    rotation: Number((8 - progress * 16).toFixed(2)),
  };
});

export const animations: Record<AnimationName, AnimationDefinition> = {
  idle: { label: "待机", frames: rowFrames(0, [280, 110, 110, 140, 140, 320]) },
  "running-right": {
    label: "向右跑",
    frames: rowFrames(1, [120, 120, 120, 120, 120, 120, 120, 220]),
  },
  "running-left": {
    label: "向左跑",
    frames: rowFrames(2, [120, 120, 120, 120, 120, 120, 120, 220]),
  },
  waving: { label: "挥手", frames: rowFrames(3, [140, 140, 140, 280]) },
  jumping: { label: "跳跃", frames: rowFrames(4, [140, 140, 140, 140, 280]) },
  failed: {
    label: "失败",
    frames: rowFrames(5, [140, 140, 140, 140, 140, 140, 140, 240]),
  },
  "web-swing": {
    label: "蜘蛛侠 4 · 吊线摆荡",
    frames: spiderManSwingFrames,
  },
  waiting: { label: "等待", frames: rowFrames(6, [150, 150, 150, 150, 150, 260]) },
  running: { label: "小跑", frames: rowFrames(7, [120, 120, 120, 120, 120, 220]) },
  review: { label: "检查", frames: rowFrames(8, [150, 150, 150, 150, 150, 280]) },
};

export const standardTriggerAnimations = [
  "idle",
  "running-right",
  "running-left",
  "waving",
  "jumping",
  "failed",
  "waiting",
  "running",
  "review",
] as const satisfies readonly AnimationName[];

export function randomTriggerAnimation(
  petId: string,
  previous?: AnimationName,
  random: () => number = Math.random,
): AnimationName {
  if (petId === "spiderman4-sticker") return "web-swing";

  const candidates = previous
    ? standardTriggerAnimations.filter((animation) => animation !== previous)
    : standardTriggerAnimations;
  const index = Math.min(
    candidates.length - 1,
    Math.max(0, Math.floor(random() * candidates.length)),
  );
  return candidates[index];
}

export function galleryPreviewAnimation(petId: string): AnimationName {
  return petId === "spiderman4-sticker" ? "web-swing" : "waving";
}
