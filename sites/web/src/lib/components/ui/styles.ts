// Class lists shared by several components, kept in one place so the pill and
// field styles stay identical everywhere.

/** Toggle pill used by RangeSlider presets and ToggleChips (Figma "6 Units"). */
export const togglePill =
  "inline-flex h-[34px] min-w-0 items-center justify-center rounded-full border border-line-strong px-2 text-label-md text-ink transition-colors hover:bg-hover aria-pressed:bg-line aria-pressed:text-strong disabled:opacity-50";

/** Bordered form field (Figma "Text field"): selects, comboboxes, text inputs. */
export const fieldBox =
  "focus-field h-10 w-full rounded-field border border-line-strong bg-surface text-body-2 text-ink placeholder:text-muted hover:border-muted disabled:border-line disabled:text-faint";

/** Floating panel surface for popovers, menus and dropdown lists. */
export const panelSurface = "rounded-[8px] border border-line bg-surface text-ink shadow-popover";

/** One row in a dropdown list or menu. */
export const listRow =
  "flex w-full cursor-pointer items-center gap-3 px-3.5 py-1.5 text-left text-body-2 text-ink outline-none data-[active=true]:bg-paper aria-disabled:cursor-not-allowed aria-disabled:text-faint";
