// Pan and zoom for the requisite graph. The graph sits in a scroll box: panning scrolls it (a drag with the mouse, a
// swipe on a touch screen, the wheel or the trackpad), and zooming scales the graph inside a sizer as big as the scaled
// graph, so the scroll box's own scrollbars and keyboard scrolling keep working. A graph smaller than the box is
// centered in it. These are the sums.

export interface Size {
  width: number;
  height: number;
}

export const MIN_ZOOM = 0.3;
export const MAX_ZOOM = 2;

export function clampZoom(zoom: number, min = MIN_ZOOM, max = MAX_ZOOM): number {
  return Math.min(max, Math.max(min, zoom));
}

/** Where the scaled graph starts in the sizer along one axis: centered when it's smaller than the box. */
export function offset(length: number, box: number): number {
  return Math.max(0, (box - length) / 2);
}

/** The zoom that shows the whole graph in the box, never above 100%. */
export function fitZoom(content: Size, box: Size, min = MIN_ZOOM): number {
  if (content.width <= 0 || content.height <= 0 || box.width <= 0 || box.height <= 0) return 1;
  return clampZoom(Math.min(box.width / content.width, box.height / content.height, 1), min);
}

/** The first zoom: the graph's width fits the box when it can, but not so small its courses get hard to read. */
export function defaultZoom(content: Size, box: Size, floor: number): number {
  if (content.width <= 0 || box.width <= 0) return 1;
  return clampZoom(Math.min(1, box.width / content.width), floor);
}

export interface Scroll {
  left: number;
  top: number;
}

/**
 * The scroll position that keeps the graph point under `point` (in the box's coordinates) in place when the zoom
 * changes from `from` to `to`.
 */
export function zoomAround(
  scroll: Scroll,
  content: Size,
  box: Size,
  point: { x: number; y: number },
  from: number,
  to: number,
): Scroll {
  const cx = (scroll.left + point.x - offset(content.width * from, box.width)) / from;
  const cy = (scroll.top + point.y - offset(content.height * from, box.height)) / from;
  return {
    left: Math.max(0, cx * to + offset(content.width * to, box.width) - point.x),
    top: Math.max(0, cy * to + offset(content.height * to, box.height) - point.y),
  };
}

/** The scroll position that puts the graph point `target` at the box's center. */
export function centerOn(
  target: { x: number; y: number },
  content: Size,
  box: Size,
  zoom: number,
): Scroll {
  return {
    left: Math.max(0, target.x * zoom + offset(content.width * zoom, box.width) - box.width / 2),
    top: Math.max(0, target.y * zoom + offset(content.height * zoom, box.height) - box.height / 2),
  };
}

/** The smallest scroll change that brings `rect` (graph coordinates) into the box with `margin` around it. */
export function reveal(
  rect: { x: number; y: number; w: number; h: number },
  scroll: Scroll,
  content: Size,
  box: Size,
  zoom: number,
  margin: number,
): Scroll {
  const axis = (
    start: number,
    length: number,
    current: number,
    contentLength: number,
    boxLength: number,
  ): number => {
    const from = start * zoom + offset(contentLength * zoom, boxLength) - margin;
    const to = (start + length) * zoom + offset(contentLength * zoom, boxLength) + margin;
    if (from < current) return Math.max(0, from);
    if (to > current + boxLength) return Math.max(0, Math.min(from, to - boxLength));
    return current;
  };
  return {
    left: axis(rect.x, rect.w, scroll.left, content.width, box.width),
    top: axis(rect.y, rect.h, scroll.top, content.height, box.height),
  };
}
