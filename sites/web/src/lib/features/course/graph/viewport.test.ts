/* oxlint-disable typescript/no-floating-promises -- node:test awaits describe and it itself */
import { describe, it } from "node:test";
import { expect } from "../../../testing.ts";
import {
  centerOn,
  clampZoom,
  defaultZoom,
  fitZoom,
  offset,
  reveal,
  zoomAround,
} from "./viewport.ts";

const content = { width: 1000, height: 600 };
const box = { width: 500, height: 400 };

describe("viewport", () => {
  it("fits and clamps the zoom", () => {
    expect(fitZoom(content, box)).toBe(0.5);
    expect(fitZoom({ width: 100, height: 100 }, box)).toBe(1);
    expect(defaultZoom(content, box, 0.7)).toBe(0.7);
    expect(defaultZoom({ width: 400, height: 2000 }, box, 0.7)).toBe(1);
    expect(clampZoom(5)).toBe(2);
  });

  it("centers a small graph and keeps the point under the pointer while zooming", () => {
    expect(offset(300, 500)).toBe(100);
    expect(offset(800, 500)).toBe(0);
    // The graph point under the pointer was (350, 250); at 2x it's at (700, 500) and still under it.
    expect(zoomAround({ left: 100, top: 50 }, content, box, { x: 250, y: 200 }, 1, 2)).toEqual({
      left: 450,
      top: 300,
    });
    expect(centerOn({ x: 500, y: 300 }, content, box, 1)).toEqual({ left: 250, top: 100 });
  });

  it("scrolls just enough to show a focused course", () => {
    const scroll = { left: 0, top: 0 };
    expect(reveal({ x: 10, y: 10, w: 50, h: 20 }, scroll, content, box, 1, 8)).toEqual(scroll);
    expect(reveal({ x: 600, y: 10, w: 100, h: 20 }, scroll, content, box, 1, 8)).toEqual({
      left: 208,
      top: 0,
    });
  });
});
