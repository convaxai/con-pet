import type { PositionMode } from "./types";

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface PositionInput {
  workArea: Rect;
  spriteWidth: number;
  spriteHeight: number;
  margin: number;
  mode: PositionMode;
  fixedX: number;
  fixedY: number;
  random?: () => number;
}

export function computePosition(input: PositionInput): { x: number; y: number } {
  if (input.mode === "fixed") {
    return { x: Math.round(input.fixedX), y: Math.round(input.fixedY) };
  }

  const random = input.random ?? Math.random;
  const minX = input.workArea.x + input.margin;
  const minY = input.workArea.y + input.margin;
  const maxX = Math.max(minX, input.workArea.x + input.workArea.width - input.spriteWidth - input.margin);
  const maxY = Math.max(minY, input.workArea.y + input.workArea.height - input.spriteHeight - input.margin);
  const horizontal = input.mode.endsWith("right") ? maxX : minX;
  const vertical = input.mode.startsWith("bottom") ? maxY : minY;

  if (input.mode === "random") {
    return {
      x: Math.round(minX + random() * (maxX - minX)),
      y: Math.round(minY + random() * (maxY - minY)),
    };
  }
  if (input.mode === "top-center") {
    return {
      x: Math.round(input.workArea.x + (input.workArea.width - input.spriteWidth) / 2),
      y: Math.round(minY),
    };
  }
  return { x: Math.round(horizontal), y: Math.round(vertical) };
}
