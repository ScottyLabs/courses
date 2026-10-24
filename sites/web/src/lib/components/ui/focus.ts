const FOCUSABLE = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled]):not([type='hidden'])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]:not([tabindex='-1'])",
  "[contenteditable='true']",
].join(",");

/** Visible, enabled elements inside `root` that take keyboard focus, in tab order. */
export function focusableWithin(root: ParentNode): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    (element) =>
      element.tabIndex >= 0 && !element.closest("[inert]") && element.getClientRects().length > 0,
  );
}

/** Keeps Tab and Shift+Tab cycling inside `root`. Call from a keydown handler. */
export function trapTab(event: KeyboardEvent, root: HTMLElement): void {
  if (event.key !== "Tab") return;
  const items = focusableWithin(root);
  const first = items.at(0);
  const last = items.at(-1);
  if (first === undefined || last === undefined) {
    event.preventDefault();
    root.focus();
    return;
  }
  const active = document.activeElement;
  if (event.shiftKey && (active === first || !root.contains(active))) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && (active === last || !root.contains(active))) {
    event.preventDefault();
    first.focus();
  }
}

/** Focuses `element` if it is still in the document; used to return focus after a panel closes. */
export function restoreFocus(element: Element | null): void {
  if (element instanceof HTMLElement && element.isConnected) element.focus({ preventScroll: true });
}
