import type { AnimationName } from "./types";

export interface AnimationDefinition {
  frames: readonly SpriteFrame[];
}

export interface SpriteFrame {
  column: number;
  row: number;
  duration: number;
  offsetX?: number;
  offsetY?: number;
  rotation?: number;
  transformOrigin?: string;
  reveal?: number;
}

const rowFrames = (row: number, durations: readonly number[]): SpriteFrame[] =>
  durations.map((duration, column) => ({ column, row, duration }));

const atlasFrame = (frameIndex: number, duration: number): SpriteFrame => ({
  column: frameIndex % 8,
  row: Math.floor(frameIndex / 8),
  duration,
});

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

const repeatedRange = (start: number, count: number, repeats = 2): number[] =>
  Array.from({ length: repeats }, () =>
    Array.from({ length: count }, (_, index) => start + index),
  ).flat();

// Cells 20-32 are official-effect frames. Each cell already contains one
// uninterrupted screen-top web plus the feet, hands, body, and oversized head.
// The overlay only reveals the complete cell; it never draws or joins a web.
const spiderManUpsideDownOrder = [
  ...Array.from({ length: 13 }, (_, index) => index),
  ...Array.from({ length: 11 }, (_, index) => 11 - index),
  ...Array.from({ length: 11 }, (_, index) => index + 2),
  ...Array.from({ length: 12 }, (_, index) => 11 - index),
];
const spiderManUpsideDownFrames: SpriteFrame[] = spiderManUpsideDownOrder.map(
  (poseIndex, index) => {
    const progress = index / (spiderManUpsideDownOrder.length - 1);
    let reveal: number;
    if (progress < 0.22) {
      const descend = progress / 0.22;
      reveal = 1 - (1 - descend) ** 3;
    } else if (progress < 0.78) {
      reveal = 1;
    } else {
      const retract = (progress - 0.78) / 0.22;
      reveal = 1 - retract ** 3;
    }
    return {
      ...atlasFrame(
        20 + poseIndex,
        index === 0 || index === spiderManUpsideDownOrder.length - 1 ? 170 : 120,
      ),
      reveal: Number(reveal.toFixed(4)),
    };
  },
);

const spiderManHeartFrames: SpriteFrame[] = repeatedRange(41, 22).map(
  (frameIndex, index) => ({
    ...atlasFrame(frameIndex, 110),
    offsetY: Math.round(Math.sin((index / 21) * Math.PI * 2) * 4),
  }),
);

const spiderManCryingFrames: SpriteFrame[] = repeatedRange(63, 20).map(
  (frameIndex, index) => ({
    ...atlasFrame(frameIndex, 120),
    offsetX: Math.round(Math.sin((index / 19) * Math.PI * 6) * 3),
  }),
);

export const animations: Record<AnimationName, AnimationDefinition> = {
  idle: { frames: rowFrames(0, [280, 110, 110, 140, 140, 320]) },
  "running-right": {
    frames: rowFrames(1, [120, 120, 120, 120, 120, 120, 120, 220]),
  },
  "running-left": {
    frames: rowFrames(2, [120, 120, 120, 120, 120, 120, 120, 220]),
  },
  waving: { frames: rowFrames(3, [140, 140, 140, 280]) },
  jumping: { frames: rowFrames(4, [140, 140, 140, 140, 280]) },
  failed: {
    frames: rowFrames(5, [140, 140, 140, 140, 140, 140, 140, 240]),
  },
  "web-swing": {
    frames: spiderManSwingFrames,
  },
  "spider-upside-down": { frames: spiderManUpsideDownFrames },
  "spider-heart": { frames: spiderManHeartFrames },
  "spider-crying": { frames: spiderManCryingFrames },
  "spider-showcase": {
    frames: [
      ...spiderManSwingFrames,
      ...spiderManHeartFrames.slice(0, 22),
      ...spiderManCryingFrames.slice(0, 20),
    ],
  },
  waiting: { frames: rowFrames(6, [150, 150, 150, 150, 150, 260]) },
  running: { frames: rowFrames(7, [120, 120, 120, 120, 120, 220]) },
  review: { frames: rowFrames(8, [150, 150, 150, 150, 150, 280]) },
};

export const spiderManTriggerAnimations = [
  "web-swing",
  "spider-upside-down",
  "spider-heart",
  "spider-crying",
] as const satisfies readonly AnimationName[];

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
  const available =
    petId === "spiderman4-sticker" ? spiderManTriggerAnimations : standardTriggerAnimations;

  const candidates = previous
    ? available.filter((animation) => animation !== previous)
    : available;
  const index = Math.min(
    candidates.length - 1,
    Math.max(0, Math.floor(random() * candidates.length)),
  );
  return candidates[index];
}

export function galleryPreviewAnimation(petId: string): AnimationName {
  return petId === "spiderman4-sticker" ? "spider-showcase" : "waving";
}
