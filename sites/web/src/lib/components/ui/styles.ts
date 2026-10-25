// Class lists shared by several components, kept in one place so the pill and
// field styles stay identical everywhere.

/** A toggle's look without its size: the border, the text and the pressed fill. */
const toggleLook =
  "inline-flex min-w-0 items-center justify-center rounded-full border border-line-strong text-label-md text-ink transition-colors hover:bg-hover aria-pressed:bg-line aria-pressed:text-strong disabled:opacity-50";

/** Toggle pill used by RangeSlider presets and ToggleChips (Figma "6 Units"). */
/** A toggle pill: a 40px tap target on touch layouts, the Figma's 34px from lg. */
export const togglePill = `${toggleLook} h-10 px-2 lg:h-[34px]`;

/** A round toggle for a single letter, such as a weekday: a 40px tap target on touch layouts, the pill's 34px from lg. */
export const toggleCircle = `${toggleLook} size-10 shrink-0 lg:size-[34px]`;

/** Bordered form field (Figma "Text field"): selects, comboboxes, text inputs. */
export const fieldBox =
  "focus-field h-10 w-full rounded-field border border-line-strong bg-surface text-body-2 text-ink placeholder:text-muted hover:border-muted disabled:border-line disabled:text-faint";

/** Floating panel surface for popovers, menus and dropdown lists. */
export const panelSurface = "rounded-[8px] border border-line bg-surface text-ink shadow-popover";

/** One row in a dropdown list or menu. */
export const listRow =
  "flex w-full cursor-pointer items-center gap-3 px-3.5 py-1.5 text-left text-body-2 text-ink outline-none data-[active=true]:bg-paper aria-disabled:cursor-not-allowed aria-disabled:text-faint";
