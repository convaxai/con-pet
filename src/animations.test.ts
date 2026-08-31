import { describe, expect, it } from "vitest";
import {
  animations,
  galleryPreviewAnimation,
  randomTriggerAnimation,
  standardTriggerAnimations,
} from "./animations";

describe("Spider-Man web swing", () => {
  const frames = animations["web-swing"].frames;

  it("plays three pendulum passes for about five seconds", () => {
    expect(frames).toHaveLength(55);
    expect(frames.reduce((total, frame) => total + frame.duration, 0)).toBe(5_310);
  });

  it("uses every complete source pose, skips the empty cell, and reverses direction", () => {
    const sourceIndexes = frames.map((frame) => frame.row * 8 + frame.column);
    expect(sourceIndexes.slice(0, 19)).toEqual(Array.from({ length: 19 }, (_, index) => index + 1));
    expect(sourceIndexes).not.toContain(0);
    expect(sourceIndexes[19]).toBe(18);
    expect(sourceIndexes[36]).toBe(1);
    expect(sourceIndexes[sourceIndexes.length - 1]).toBe(19);
  });

  it("adds a restrained top-anchored swing arc", () => {
    expect(frames[0]).toMatchObject({ offsetX: -62, offsetY: 0, rotation: 8 });
    expect(frames[18]).toMatchObject({ offsetX: 62, offsetY: 0, rotation: -8 });
    expect(Math.max(...frames.map((frame) => frame.offsetY ?? 0))).toBe(18);
  });
});

describe("gallery animation previews", () => {
  it("uses the custom swing for Spider-Man and waving for standard pets", () => {
    expect(galleryPreviewAnimation("spiderman4-sticker")).toBe("web-swing");
    expect(galleryPreviewAnimation("pathlight")).toBe("waving");
  });
});

describe("random trigger animations", () => {
  it("covers every standard Codex state", () => {
    expect(standardTriggerAnimations).toEqual([
      "idle",
      "running-right",
      "running-left",
      "waving",
      "jumping",
      "failed",
      "waiting",
      "running",
      "review",
    ]);
    expect(randomTriggerAnimation("pathlight", undefined, () => 0)).toBe("idle");
    expect(randomTriggerAnimation("pathlight", undefined, () => 0.999)).toBe("review");
  });

  it("avoids an immediate repeat and preserves custom Spider-Man motion", () => {
    expect(randomTriggerAnimation("pathlight", "idle", () => 0)).toBe("running-right");
    expect(randomTriggerAnimation("spiderman4-sticker", "web-swing", () => 0)).toBe(
      "web-swing",
    );
  });
});
