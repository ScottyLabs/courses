// The requisite graph's pan and zoom (./viewport.ts has the sums). The graph sits in a scroll box: dragging the
// background with the mouse, swiping, the wheel and the scrollbars pan it; ctrl or a trackpad pinch (a ctrl wheel),
// a two-finger pinch and the zoom buttons zoom it around the pointer. Until the student pans or zooms, the zoom fits
// the graph's width (within reason) and follows it as statuses and unlocks load.
import { flushSync } from "svelte";
import type { Rect } from "./geometry.ts";
import {
  centerOn,
  clampZoom,
  defaultZoom,
  fitZoom,
  offset,
  reveal,
  zoomAround,
  type Scroll,
  type Size,
} from "./viewport.ts";

export interface PanZoomSource {
  /** The graph's size at 100%. */
  content: () => Size;
  /** The graph point the first view centers: the middle course. */
  home: () => { x: number; y: number };
  /** Center a graph smaller than the box (wide screens); else it starts at the left. */
  centered: () => boolean;
  /** The smallest first zoom. */
  floor: () => number;
}

interface Pinch {
  distance: number;
  zoom: number;
}

function spread(touches: TouchList): { distance: number; x: number; y: number } | null {
  const a = touches[0];
  const b = touches[1];
  if (a === undefined || b === undefined) return null;
  const distance = Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
  return { distance, x: (a.clientX + b.clientX) / 2, y: (a.clientY + b.clientY) / 2 };
}

export class PanZoom {
  viewport = $state<HTMLElement | undefined>();
  boxWidth = $state(0);
  boxHeight = $state(0);
  dragging = $state(false);
  /** Untouched since the first view, so the view follows the graph as it loads. */
  pristine = true;
  #chosen = $state<number | null>(null);
  #drag: { x: number; y: number; left: number; top: number; id: number } | null = null;
  #pinch: Pinch | null = null;
  readonly #source: PanZoomSource;

  constructor(source: PanZoomSource) {
    this.#source = source;
  }

  /** The first view's zoom. */
  get base(): number {
    return defaultZoom(
      this.#source.content(),
      { width: this.boxWidth, height: 0 },
      this.#source.floor(),
    );
  }

  get zoom(): number {
    return this.#chosen ?? this.base;
  }

  /** Where the scaled graph starts in the scroll box's content. */
  get left(): number {
    return this.#source.centered()
      ? offset(this.#source.content().width * this.zoom, this.boxWidth)
      : 0;
  }

  get top(): number {
    return offset(this.#source.content().height * this.zoom, this.boxHeight);
  }

  #box(): Size {
    return { width: this.boxWidth, height: this.boxHeight };
  }

  #scroll(): Scroll {
    return { left: this.viewport?.scrollLeft ?? 0, top: this.viewport?.scrollTop ?? 0 };
  }

  scrollTo(scroll: Scroll): void {
    this.viewport?.scrollTo({ left: scroll.left, top: scroll.top, behavior: "instant" });
  }

  /** Zooms around `point` in the scroll box (its middle by default). */
  set(next: number, point: { x: number; y: number } | null = null): void {
    if (this.viewport === undefined) return;
    const to = clampZoom(next);
    const at = point ?? { x: this.boxWidth / 2, y: this.boxHeight / 2 };
    const scroll = zoomAround(
      this.#scroll(),
      this.#source.content(),
      this.#box(),
      at,
      this.zoom,
      to,
    );
    this.pristine = false;
    this.#chosen = to;
    flushSync();
    this.scrollTo(scroll);
  }

  #zoomAtClient(next: number, clientX: number, clientY: number): void {
    const rect = this.viewport?.getBoundingClientRect();
    if (rect !== undefined) this.set(next, { x: clientX - rect.left, y: clientY - rect.top });
  }

  /** Centers the middle course. */
  home(): void {
    this.scrollTo(centerOn(this.#source.home(), this.#source.content(), this.#box(), this.zoom));
  }

  /** The first view again. */
  reset(): void {
    this.#chosen = null;
    this.pristine = true;
    flushSync();
    this.home();
  }

  /** The whole graph in view. */
  fit(): void {
    const content = this.#source.content();
    this.pristine = false;
    this.#chosen = fitZoom(content, this.#box(), 0.2);
    flushSync();
    this.scrollTo(
      centerOn({ x: content.width / 2, y: content.height / 2 }, content, this.#box(), this.zoom),
    );
  }

  /** Where a graph rectangle's corner is in the scroll box's content. */
  position(rect: Rect): { x: number; y: number } {
    return { x: rect.x * this.zoom + this.left, y: rect.y * this.zoom + this.top };
  }

  /** Scrolls just enough to show a graph rectangle. */
  reveal(rect: Rect): void {
    this.scrollTo(reveal(rect, this.#scroll(), this.#source.content(), this.#box(), this.zoom, 24));
  }

  /** Runs `change`, which reshapes the graph, keeping the zoom and what `find` finds where they were on screen. */
  keep(find: () => Rect | null, change: () => void): void {
    const found = find();
    const before = found === null ? null : this.position(found);
    this.pristine = false;
    this.#chosen = this.zoom;
    change();
    flushSync();
    const moved = find();
    if (before === null || moved === null) return;
    const after = this.position(moved);
    const scroll = this.#scroll();
    this.scrollTo({ left: scroll.left + after.x - before.x, top: scroll.top + after.y - before.y });
  }

  // --- Input ------------------------------------------------------------------------------------------------------

  readonly #onWheel = (event: WheelEvent): void => {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    const delta = event.deltaMode === 1 ? event.deltaY * 16 : event.deltaY;
    this.#zoomAtClient(
      this.zoom * Math.min(1.25, Math.max(0.8, Math.exp(-delta * 0.01))),
      event.clientX,
      event.clientY,
    );
  };

  readonly #onPointerDown = (event: PointerEvent): void => {
    this.pristine = false;
    const el = this.viewport;
    if (event.pointerType !== "mouse" || event.button !== 0 || el === undefined) return;
    if (event.target instanceof Element && event.target.closest("a, button") !== null) return;
    this.#drag = {
      x: event.clientX,
      y: event.clientY,
      left: el.scrollLeft,
      top: el.scrollTop,
      id: event.pointerId,
    };
    el.setPointerCapture(event.pointerId);
    this.dragging = true;
    event.preventDefault();
  };

  readonly #onPointerMove = (event: PointerEvent): void => {
    const drag = this.#drag;
    if (drag === null || event.pointerId !== drag.id) return;
    this.scrollTo({
      left: drag.left - (event.clientX - drag.x),
      top: drag.top - (event.clientY - drag.y),
    });
  };

  readonly #onPointerUp = (event: PointerEvent): void => {
    if (this.#drag === null || event.pointerId !== this.#drag.id) return;
    this.viewport?.releasePointerCapture(event.pointerId);
    this.#drag = null;
    this.dragging = false;
  };

  readonly #onTouchStart = (event: TouchEvent): void => {
    this.pristine = false;
    const s = event.touches.length === 2 ? spread(event.touches) : null;
    this.#pinch = s === null || s.distance === 0 ? null : { distance: s.distance, zoom: this.zoom };
  };

  readonly #onTouchMove = (event: TouchEvent): void => {
    const pinch = this.#pinch;
    const s = pinch === null || event.touches.length !== 2 ? null : spread(event.touches);
    if (pinch === null || s === null) return;
    if (event.cancelable) event.preventDefault();
    this.#zoomAtClient(pinch.zoom * (s.distance / pinch.distance), s.x, s.y);
  };

  readonly #onTouchEnd = (event: TouchEvent): void => {
    if (event.touches.length < 2) this.#pinch = null;
  };

  /** Listens on the scroll box; returns the function that stops. */
  attach(el: HTMLElement): () => void {
    const controller = new AbortController();
    const { signal } = controller;
    el.addEventListener("wheel", this.#onWheel, { passive: false, signal });
    el.addEventListener("pointerdown", this.#onPointerDown, { signal });
    el.addEventListener("pointermove", this.#onPointerMove, { signal });
    el.addEventListener("pointerup", this.#onPointerUp, { signal });
    el.addEventListener("pointercancel", this.#onPointerUp, { signal });
    el.addEventListener("touchstart", this.#onTouchStart, { passive: true, signal });
    el.addEventListener("touchmove", this.#onTouchMove, { passive: false, signal });
    el.addEventListener("touchend", this.#onTouchEnd, { passive: true, signal });
    el.addEventListener("touchcancel", this.#onTouchEnd, { passive: true, signal });
    return () => {
      controller.abort();
    };
  }
}
