import { describe, expect, it } from "vitest";
import {
  animations,
  galleryPreviewAnimation,
  randomTriggerAnimation,
  spiderManTriggerAnimations,
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

describe("Spider-Man extended effects", () => {
  it("uses complete official-effect frames for the upside-down entrance", () => {
    const upsideDown = animations["spider-upside-down"].frames;
    const heart = animations["spider-heart"].frames;
    const crying = animations["spider-crying"].frames;

    expect(upsideDown).toHaveLength(47);
    expect(heart).toHaveLength(44);
    expect(crying).toHaveLength(40);
    expect(new Set(upsideDown.map((frame) => frame.row * 8 + frame.column))).toEqual(
      new Set(Array.from({ length: 13 }, (_, index) => index + 20)),
    );
    expect(new Set(heart.map((frame) => frame.row * 8 + frame.column))).toEqual(
      new Set(Array.from({ length: 22 }, (_, index) => index + 41)),
    );
    expect(new Set(crying.map((frame) => frame.row * 8 + frame.column))).toEqual(
      new Set(Array.from({ length: 20 }, (_, index) => index + 63)),
    );
  });

  it("keeps every effect visible for roughly four to five seconds", () => {
    expect(
      animations["spider-upside-down"].frames.reduce(
        (total, frame) => total + frame.duration,
        0,
      ),
    ).toBe(5_740);
    expect(
      animations["spider-heart"].frames.reduce((total, frame) => total + frame.duration, 0),
    ).toBe(4_840);
    expect(
      animations["spider-crying"].frames.reduce((total, frame) => total + frame.duration, 0),
    ).toBe(4_800);
  });

  it("reveals and retracts the intact web-and-hero frame from the screen top", () => {
    const frames = animations["spider-upside-down"].frames;
    expect(frames[0]).toMatchObject({
      column: 4,
      row: 2,
      reveal: 0,
    });
    expect(Math.max(...frames.map((frame) => frame.reveal ?? 0))).toBe(1);
    expect(frames[frames.length - 1]).toMatchObject({
      reveal: 0,
    });
    expect(new Set(frames.map((frame) => frame.row * 8 + frame.column))).toEqual(
      new Set(Array.from({ length: 13 }, (_, index) => index + 20)),
    );
    expect(frames.every((frame) => frame.rotation === undefined)).toBe(true);
    expect(frames.every((frame) => frame.offsetY === undefined)).toBe(true);
  });
});

describe("gallery animation previews", () => {
  it("shows several Spider-Man effects and waving for standard pets", () => {
    expect(galleryPreviewAnimation("spiderman4-sticker")).toBe("spider-showcase");
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

  it("randomizes every Spider-Man effect without an immediate repeat", () => {
    expect(spiderManTriggerAnimations).toEqual([
      "web-swing",
      "spider-upside-down",
      "spider-heart",
      "spider-crying",
    ]);
    expect(randomTriggerAnimation("pathlight", "idle", () => 0)).toBe("running-right");
    expect(randomTriggerAnimation("spiderman4-sticker", "web-swing", () => 0)).toBe(
      "spider-upside-down",
    );
    expect(randomTriggerAnimation("spiderman4-sticker", undefined, () => 0)).toBe("web-swing");
    expect(randomTriggerAnimation("spiderman4-sticker", undefined, () => 0.999)).toBe(
      "spider-crying",
    );
  });
});
