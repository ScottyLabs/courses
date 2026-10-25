// Arrow keys in the requisite graph move between what's on it by where things are drawn.
import type { Rect } from "./geometry.ts";

export type Direction = "left" | "right" | "up" | "down";

/**
 * The item an arrow key moves to from `from`: the nearest one whose center lies that way, favouring ones in line
 * (a sideways step counts twice as far as a straight one, four times going up or down). Null when there's none.
 */
export function nextInDirection<T extends Rect & { key: string }>(
  items: readonly T[],
  from: T,
  direction: Direction,
): T | null {
  const cx = from.x + from.w / 2;
  const cy = from.y + from.h / 2;
  const vertical = direction === "up" || direction === "down";
  let best: T | null = null;
  let bestScore = Infinity;
  for (const item of items) {
    if (item.key === from.key) continue;
    const dx = item.x + item.w / 2 - cx;
    const dy = item.y + item.h / 2 - cy;
    const along = { left: -dx, right: dx, up: -dy, down: dy }[direction];
    if (along <= 1) continue;
    const score = along + (vertical ? Math.abs(dx) * 4 : Math.abs(dy) * 2);
    if (score < bestScore) {
      bestScore = score;
      best = item;
    }
  }
  return best;
}
