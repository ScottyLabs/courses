// A rail's scroll box always keeps room for its scrollbar (`scrollbar-gutter: stable`), and its children reach under
// that room with a negative right margin of `--sb`, so the rail's content is as wide, and lines up the same, whether
// or not the rail scrolls; the scrollbar sits over the content's right padding.

/** The attachment for such a scroll box: sets `--sb` to the gutter's width (0 where scrollbars overlay content). */
export function stableGutter(node: HTMLElement): () => void {
  // The box has no border, so all it adds to its inner width is the gutter.
  function measure(): void {
    const gutter = node.offsetWidth === 0 ? 0 : Math.max(0, node.offsetWidth - node.clientWidth);
    node.style.setProperty("--sb", `${gutter}px`);
  }
  // The children overflow the box sideways by the gutter, which is hidden; keep anything (focus moving to a control,
  // find in page) from scrolling the box sideways.
  function unscroll(): void {
    if (node.scrollLeft !== 0) node.scrollLeft = 0;
  }
  measure();
  node.addEventListener("scroll", unscroll, { passive: true });
  // The rail opening, closing or resizing, and page zoom (which changes the scrollbar's width in CSS pixels).
  const observer = new ResizeObserver(measure);
  observer.observe(node);
  window.addEventListener("resize", measure);
  return () => {
    observer.disconnect();
    node.removeEventListener("scroll", unscroll);
    window.removeEventListener("resize", measure);
  };
}

/** The scroll box's classes: the stable gutter, no sideways scrolling, and children reaching under the gutter. */
export const STABLE_GUTTER =
  "[scrollbar-gutter:stable] overflow-x-hidden *:mr-[calc(-1*var(--sb,0px))]";
