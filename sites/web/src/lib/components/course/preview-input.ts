// When course previews open and close (CoursePreviewLayer): mouse hover with Courseweave's delays, keyboard focus,
// and press-and-hold on touch screens, whose following click is swallowed so the press doesn't follow the link.
import type { PreviewTarget } from "./preview.ts";

const OPEN_DELAY = 450;
/** After a card was open, neighbouring codes open almost at once (like native tooltips). */
const WARM_DELAY = 90;
const CLOSE_DELAY = 160;
/** How long a touch must rest on a course link to open its card. */
const PRESS_DELAY = 450;
/** A touch that moves further than this (pixels) is a scroll, not a press. */
const PRESS_SLOP = 10;

/** A course link or marked element, what it previews and where "Open course" goes. */
export interface PreviewAnchor {
  anchor: Element;
  target: PreviewTarget;
  href: string;
}

/** hover: a tooltip by the anchor (mouse and keyboard); touch: a card at the bottom of the screen. */
export type PreviewMode = "hover" | "touch";

/** What the input needs from the layer. */
export interface PreviewHost {
  /** The course link or marked element at a node, or null when it has nothing to preview. */
  find: (node: EventTarget | null) => PreviewAnchor | null;
  /** Whether a node is inside the open card. */
  inCard: (node: EventTarget | null) => boolean;
  /** The open card's anchor and mode, if one is open. */
  current: () => { anchor: Element; mode: PreviewMode } | null;
  open: (found: PreviewAnchor, mode: PreviewMode) => void;
  close: () => void;
}

export class PreviewInput {
  readonly #host: PreviewHost;
  #openTimer: ReturnType<typeof setTimeout> | undefined;
  #closeTimer: ReturnType<typeof setTimeout> | undefined;
  #pressTimer: ReturnType<typeof setTimeout> | undefined;
  #press: { x: number; y: number } | null = null;
  /** The link a long press opened a card for: the click that follows the press must not follow it. */
  #swallow: Element | null = null;
  #swallowUntil = 0;
  #warmUntil = 0;
  #lastPointer = "mouse";

  constructor(host: PreviewHost) {
    this.#host = host;
  }

  /** Listens on the document; returns the function that stops. */
  install(): () => void {
    const controller = new AbortController();
    const { signal } = controller;
    document.addEventListener("pointerover", this.#onOver, { signal });
    document.addEventListener("pointerout", this.#onOut, { signal });
    document.addEventListener("focusin", this.#onFocusIn, { signal });
    document.addEventListener("focusout", this.#onFocusOut, { signal });
    document.addEventListener("keydown", this.#onKey, { signal });
    document.addEventListener("pointerdown", this.#onDown, { capture: true, signal });
    document.addEventListener("pointermove", this.#onMove, { passive: true, signal });
    document.addEventListener("pointerup", this.#cancelPress, { capture: true, signal });
    document.addEventListener("pointercancel", this.#cancelPress, { capture: true, signal });
    document.addEventListener("click", this.#onClick, { capture: true, signal });
    document.addEventListener("contextmenu", this.#onContextMenu, { capture: true, signal });
    window.addEventListener("scroll", this.#onScroll, { capture: true, passive: true, signal });
    window.addEventListener("resize", this.#hideNow, { signal });
    return () => {
      this.#clear();
      this.#cancelPress();
      controller.abort();
    };
  }

  #clear(): void {
    clearTimeout(this.#openTimer);
    clearTimeout(this.#closeTimer);
  }

  readonly #hideNow = (): void => {
    this.#clear();
    this.#host.close();
  };

  #scheduleOpen(found: PreviewAnchor): void {
    this.#clear();
    const current = this.#host.current();
    if (current?.anchor === found.anchor) return;
    const delay = current !== null || Date.now() < this.#warmUntil ? WARM_DELAY : OPEN_DELAY;
    this.#openTimer = setTimeout(() => {
      this.#host.open(found, "hover");
    }, delay);
  }

  #scheduleClose(): void {
    this.#clear();
    this.#closeTimer = setTimeout(() => {
      if (this.#host.current() !== null) this.#warmUntil = Date.now() + 600;
      this.#host.close();
    }, CLOSE_DELAY);
  }

  #touchOpen(): boolean {
    return this.#host.current()?.mode === "touch";
  }

  readonly #cancelPress = (): void => {
    clearTimeout(this.#pressTimer);
    this.#press = null;
  };

  readonly #onOver = (event: PointerEvent): void => {
    if (event.pointerType !== "mouse") return;
    if (this.#host.inCard(event.target)) {
      clearTimeout(this.#closeTimer);
      return;
    }
    const found = this.#host.find(event.target);
    if (found !== null && !this.#touchOpen()) this.#scheduleOpen(found);
  };

  readonly #onOut = (event: PointerEvent): void => {
    if (event.pointerType !== "mouse" || this.#touchOpen()) return;
    const from =
      this.#host.find(event.target)?.anchor ??
      (this.#host.inCard(event.target) ? this.#host.current()?.anchor : null);
    if (from === null || from === undefined) return;
    const to = event.relatedTarget;
    if (to instanceof Node && (from.contains(to) || this.#host.inCard(to))) return;
    this.#scheduleClose();
  };

  readonly #onFocusIn = (event: FocusEvent): void => {
    if (!(event.target instanceof Element) || !event.target.matches(":focus-visible")) return;
    const found = this.#host.find(event.target);
    if (found !== null) this.#scheduleOpen(found);
  };

  readonly #onFocusOut = (event: FocusEvent): void => {
    if (!this.#touchOpen() && this.#host.find(event.target) !== null) this.#scheduleClose();
  };

  readonly #onKey = (event: KeyboardEvent): void => {
    if (event.key === "Escape" && this.#host.current() !== null) this.#hideNow();
  };

  readonly #onDown = (event: PointerEvent): void => {
    this.#lastPointer = event.pointerType;
    if (this.#host.inCard(event.target)) return;
    // Pressing anything else acts on it; the card would only be in the way.
    this.#hideNow();
    if (event.pointerType === "mouse" || !event.isPrimary) return;
    const found = this.#host.find(event.target);
    if (found === null) return;
    this.#press = { x: event.clientX, y: event.clientY };
    clearTimeout(this.#pressTimer);
    this.#pressTimer = setTimeout(() => {
      this.#press = null;
      this.#swallow = found.anchor;
      this.#swallowUntil = Date.now() + 1500;
      this.#host.open(found, "touch");
    }, PRESS_DELAY);
  };

  readonly #onMove = (event: PointerEvent): void => {
    const press = this.#press;
    if (press !== null && Math.hypot(event.clientX - press.x, event.clientY - press.y) > PRESS_SLOP)
      this.#cancelPress();
  };

  readonly #onClick = (event: MouseEvent): void => {
    const swallow = this.#swallow;
    if (swallow === null) return;
    this.#swallow = null;
    const target = event.target;
    if (Date.now() < this.#swallowUntil && target instanceof Node && swallow.contains(target)) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  };

  // A long press on a link opens the browser's link menu; the card takes its place on course links.
  readonly #onContextMenu = (event: MouseEvent): void => {
    if (this.#lastPointer === "mouse") return;
    if ((this.#press !== null || this.#touchOpen()) && this.#host.find(event.target) !== null)
      event.preventDefault();
  };

  readonly #onScroll = (event: Event): void => {
    if (this.#host.current() !== null && !this.#host.inCard(event.target)) this.#hideNow();
    this.#cancelPress();
  };
}
