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
  webAnchorY?: number;
}

const rowFrames = (row: number, durations: readonly number[]): SpriteFrame[] =>
  durations.map((duration, column) => ({ column, row, duration }));

const atlasFrame = (frameIndex: number, duration: number): SpriteFrame => ({
  column: frameIndex % 8,
  row: Math.floor(frameIndex / 8),
  duration,
});

export function webLineLength(frame: SpriteFrame, scale: number): number {
  if (frame.webAnchorY === undefined) return 0;
  return Math.max(0, (frame.offsetY ?? 0) + frame.webAnchorY * scale);
}

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

// Reuse the complete body-and-wave sequence from the swing sticker. Cells
// 20-40 are close-up reaction heads and made the upside-down entrance look
// detached. The forward/reverse/forward order keeps the waving hand fluid.
const spiderManUpsideDownOrder = [
  ...Array.from({ length: 18 }, (_, index) => index + 2),
  ...Array.from({ length: 17 }, (_, index) => 18 - index),
  ...Array.from({ length: 17 }, (_, index) => index + 3),
];
const spiderManUpsideDownFrames: SpriteFrame[] = spiderManUpsideDownOrder.map(
  (frameIndex, index) => {
    const progress = index / (spiderManUpsideDownOrder.length - 1);
    const hiddenY = -360;
    const hangingY = 128;
    let offsetY: number;
    if (progress < 0.28) {
      const descend = progress / 0.28;
      offsetY = hiddenY + (1 - (1 - descend) ** 3) * (hangingY - hiddenY);
    } else if (progress < 0.72) {
      const bob = (progress - 0.28) / 0.44;
      offsetY = hangingY + Math.sin(bob * Math.PI * 4) * 8;
    } else {
      const retract = (progress - 0.72) / 0.28;
      offsetY = hangingY - retract ** 3 * (hangingY - hiddenY);
    }
    return {
      ...atlasFrame(
        frameIndex,
        index === 0 || index === spiderManUpsideDownOrder.length - 1 ? 170 : 105,
      ),
      offsetY: Math.round(offsetY),
      rotation: 180,
      transformOrigin: "50% 50%",
      // The complete inverted body starts about 32 source pixels below the
      // rotated cell edge. Terminate the web there rather than across the face.
      webAnchorY: 32,
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
