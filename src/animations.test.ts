import { describe, expect, it } from "vitest";
import { animations } from "./animations";

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
