import { describe, expect, it } from "vitest";
import { computePosition } from "./positioning";

const base = {
  workArea: { x: -100, y: 20, width: 1000, height: 800 },
  spriteWidth: 200,
  spriteHeight: 200,
  margin: 20,
  fixedX: 0,
  fixedY: 0,
};

describe("computePosition", () => {
  it("uses monitor work area for corners", () => {
    expect(computePosition({ ...base, mode: "bottom-right" })).toEqual({ x: 680, y: 600 });
  });

  it("supports deterministic random placement", () => {
    expect(computePosition({ ...base, mode: "random", random: () => 0.5 })).toEqual({
      x: 300,
      y: 320,
    });
  });

  it("centers a top-anchored animation on the selected monitor", () => {
    expect(computePosition({ ...base, mode: "top-center", margin: 0 })).toEqual({
      x: 300,
      y: 20,
    });
  });

  it("honors absolute fixed coordinates", () => {
    expect(
      computePosition({ ...base, mode: "fixed", fixedX: -321.2, fixedY: 98.8 }),
    ).toEqual({ x: -321, y: 99 });
  });
});
