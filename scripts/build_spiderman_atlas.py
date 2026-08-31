#!/usr/bin/env python3
"""Append official animated-sticker GIFs to the bundled Spider-Man atlas.

The existing wire-swing occupies cells 0-19. The three GIF inputs are packed
into the remaining cells of an 8x11 compatible Codex Pet atlas:

  20-40  upside-down entrance source
  41-62  heart
  63-82  crying

Only edge-connected near-white pixels are removed, so enclosed white artwork
(eyes, highlights, and hearts) stays opaque.
"""

from __future__ import annotations

import argparse
import math
from collections import deque
from pathlib import Path

from PIL import Image, ImageSequence


COLUMNS = 8
ROWS = 11
CELL_WIDTH = 192
CELL_HEIGHT = 208
ATLAS_SIZE = (COLUMNS * CELL_WIDTH, ROWS * CELL_HEIGHT)
MARGIN = 5


def color_distance(left: tuple[int, int, int], right: tuple[int, int, int]) -> float:
    return math.sqrt(sum((a - b) ** 2 for a, b in zip(left, right)))


def remove_edge_background(source: Image.Image, threshold: float = 42) -> Image.Image:
    image = source.convert("RGBA")
    width, height = image.size
    pixels = image.load()
    background = pixels[0, 0][:3]
    queue: deque[tuple[int, int]] = deque()
    visited = bytearray(width * height)

    for x in range(width):
        queue.append((x, 0))
        queue.append((x, height - 1))
    for y in range(height):
        queue.append((0, y))
        queue.append((width - 1, y))

    while queue:
        x, y = queue.popleft()
        index = y * width + x
        if visited[index]:
            continue
        visited[index] = 1
        red, green, blue, _alpha = pixels[x, y]
        if color_distance((red, green, blue), background) > threshold:
            continue
        pixels[x, y] = (red, green, blue, 0)
        if x > 0:
            queue.append((x - 1, y))
        if x + 1 < width:
            queue.append((x + 1, y))
        if y > 0:
            queue.append((x, y - 1))
        if y + 1 < height:
            queue.append((x, y + 1))

    return image


def decode_gif(path: Path) -> list[Image.Image]:
    with Image.open(path) as image:
        return [remove_edge_background(frame.convert("RGBA")) for frame in ImageSequence.Iterator(image)]


def normalize_sequence(frames: list[Image.Image]) -> list[Image.Image]:
    boxes = [frame.getbbox() for frame in frames]
    visible = [box for box in boxes if box is not None]
    if not visible:
        raise ValueError("animation contains no visible frames")
    union = (
        min(box[0] for box in visible),
        min(box[1] for box in visible),
        max(box[2] for box in visible),
        max(box[3] for box in visible),
    )
    width = union[2] - union[0]
    height = union[3] - union[1]
    scale = min((CELL_WIDTH - MARGIN * 2) / width, (CELL_HEIGHT - MARGIN * 2) / height)
    output_size = (max(1, round(width * scale)), max(1, round(height * scale)))
    normalized: list[Image.Image] = []
    for frame in frames:
        sprite = frame.crop(union).resize(output_size, Image.Resampling.LANCZOS)
        cell = Image.new("RGBA", (CELL_WIDTH, CELL_HEIGHT), (0, 0, 0, 0))
        cell.alpha_composite(
            sprite,
            ((CELL_WIDTH - sprite.width) // 2, (CELL_HEIGHT - sprite.height) // 2),
        )
        normalized.append(cell)
    return normalized


def append_sequence(atlas: Image.Image, frames: list[Image.Image], start: int) -> None:
    for offset, frame in enumerate(frames):
        index = start + offset
        column = index % COLUMNS
        row = index // COLUMNS
        atlas.alpha_composite(frame, (column * CELL_WIDTH, row * CELL_HEIGHT))


def validate_output(atlas: Image.Image) -> None:
    if atlas.size != ATLAS_SIZE:
        raise ValueError(f"output atlas must be {ATLAS_SIZE}, got {atlas.size}")
    alpha = atlas.getchannel("A")
    for index in range(COLUMNS * ROWS):
        column = index % COLUMNS
        row = index // COLUMNS
        cell = alpha.crop(
            (
                column * CELL_WIDTH,
                row * CELL_HEIGHT,
                (column + 1) * CELL_WIDTH,
                (row + 1) * CELL_HEIGHT,
            )
        )
        visible = sum(cell.histogram()[1:])
        if index < 83 and visible < 50:
            raise ValueError(f"used atlas cell {index} is empty or too sparse ({visible} pixels)")
        if index >= 83 and visible != 0:
            raise ValueError(f"unused atlas cell {index} is not transparent ({visible} pixels)")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", type=Path, required=True)
    parser.add_argument("--upside-down", type=Path, required=True)
    parser.add_argument("--heart", type=Path, required=True)
    parser.add_argument("--crying", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    sequences = [
        (20, args.upside_down, 21),
        (41, args.heart, 22),
        (63, args.crying, 20),
    ]
    with Image.open(args.base) as base:
        if base.size not in {(1536, 1872), ATLAS_SIZE}:
            raise ValueError(
                f"base atlas must be 1536x1872 or {ATLAS_SIZE[0]}x{ATLAS_SIZE[1]}, got {base.size}"
            )
        base = base.convert("RGBA")
        atlas = Image.new("RGBA", ATLAS_SIZE, (0, 0, 0, 0))
        for index in range(20):
            column = index % COLUMNS
            row = index // COLUMNS
            left = column * CELL_WIDTH
            top = row * CELL_HEIGHT
            atlas.alpha_composite(
                base.crop((left, top, left + CELL_WIDTH, top + CELL_HEIGHT)),
                (left, top),
            )

    for start, path, expected_frames in sequences:
        frames = normalize_sequence(decode_gif(path))
        if len(frames) != expected_frames:
            raise ValueError(f"{path} must have {expected_frames} frames, got {len(frames)}")
        append_sequence(atlas, frames, start)

    validate_output(atlas)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(args.output, optimize=True)
    print(f"wrote {args.output.resolve()} ({atlas.width}x{atlas.height})")


if __name__ == "__main__":
    main()
