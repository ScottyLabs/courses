// Positions popover-API panels (which render in the top layer, above dialogs
// and outside any overflow clipping) next to their anchor element.

export type Placement = "bottom-start" | "bottom-end" | "bottom" | "top-start" | "top-end" | "top";

export interface FloatingOptions {
  placement?: Placement;
  /** Gap between the anchor and the panel, in pixels. */
  offset?: number;
  /** Make the panel at least as wide as the anchor (dropdown lists). */
  matchWidth?: boolean;
}

const MARGIN = 8;

function clamp(value: number, low: number, high: number): number {
  return Math.min(Math.max(value, low), Math.max(low, high));
}

export function place(anchor: Element, panel: HTMLElement, options: FloatingOptions = {}): void {
  const { placement = "bottom-start", offset = 6, matchWidth = false } = options;
  const rect = anchor.getBoundingClientRect();
  if (matchWidth) panel.style.minWidth = `${rect.width}px`;
  const box = panel.getBoundingClientRect();
  const viewportWidth = document.documentElement.clientWidth;
  const viewportHeight = window.innerHeight;
  const [preferred, align = "center"] = placement.split("-");

  const below = viewportHeight - rect.bottom - offset - MARGIN;
  const above = rect.top - offset - MARGIN;
  let side = preferred;
  if (side === "bottom" && box.height > below && above > below) side = "top";
  else if (side === "top" && box.height > above && below > above) side = "bottom";

  let left = rect.left + (rect.width - box.width) / 2;
  if (align === "start") left = rect.left;
  else if (align === "end") left = rect.right - box.width;
  left = clamp(left, MARGIN, viewportWidth - MARGIN - box.width);

  const available = side === "bottom" ? below : above;
  const height = Math.min(box.height, available);
  const top = side === "bottom" ? rect.bottom + offset : rect.top - offset - height;

  panel.style.position = "fixed";
  panel.style.left = `${Math.round(left)}px`;
  panel.style.top = `${Math.round(top)}px`;
  panel.style.setProperty("--floating-available", `${Math.max(120, Math.floor(available))}px`);
  panel.dataset.side = side;
}

/** Places `panel` now and whenever the page scrolls or resizes. Returns a cleanup function. */
export function follow(
  anchor: Element,
  panel: HTMLElement,
  options: FloatingOptions = {},
): () => void {
  let frame = 0;
  const update = (): void => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      place(anchor, panel, options);
    });
  };
  place(anchor, panel, options);
  window.addEventListener("scroll", update, { capture: true, passive: true });
  window.addEventListener("resize", update, { passive: true });
  const observer = new ResizeObserver(update);
  observer.observe(anchor);
  observer.observe(panel);
  return () => {
    cancelAnimationFrame(frame);
    window.removeEventListener("scroll", update, { capture: true });
    window.removeEventListener("resize", update);
    observer.disconnect();
  };
}

/** Shows a `popover="manual"` element, raising it above anything shown before. */
export function showPanel(panel: HTMLElement): void {
  if (panel.matches(":popover-open")) panel.hidePopover();
  panel.showPopover();
}

export function hidePanel(panel: HTMLElement | undefined): void {
  if (panel?.matches(":popover-open") === true) panel.hidePopover();
}
