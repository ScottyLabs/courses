# Website design system

The website follows the
[Figma redesign](https://www.figma.com/design/vsvdhGC2QP5fsvG0Pl7W6v/CMUCourses-Redesign--summer-25---Copy-?node-id=502-10393).
Its tokens live in `sites/web/src/routes/layout.css`, its components in `sites/web/src/lib/components`, and every
primitive is on show in both themes at `/dev/ui` in development builds. [Decision 30](decisions/0030-build-ui-primitives-on-native-elements.md)
explains how overlays and form controls are built, and [decision 31](decisions/0031-scroll-the-page-with-a-sticky-rail.md)
how pages scroll.

## Tokens

Colors are Tailwind theme variables, so `bg-surface`, `text-muted` or `border-line` work anywhere. Never hard-code a
color: every token has a dark value.

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| `paper` | `#f0f4f8` | `#0e151c` | Page background, secondary buttons, the desktop header |
| `surface` | `#ffffff` | `#16202a` | Cards, rail, toolbar band, dropdowns |
| `surface-2` | `#f0f4f8` | `#1c2833` | Wells inside cards (FCE aggregate, tables) |
| `ink` | `#4a5662` | `#d3dce5` | Primary text and icons |
| `strong` | `#1e1e1e` | `#f2f5f8` | Page titles, chip text, the inverted "Filter 3" pill |
| `muted` | `#7a8fa3` | `#91a4b7` | Secondary text, department names, inactive navigation |
| `faint` | `#aebdcc` | `#5c6f81` | Disabled text |
| `line` | `#d9e1e7` | `#283541` | Dividers, band borders, selected rows and pills |
| `line-strong` | `#9eb1c2` | `#4b5e70` | Field, pill and search-underline borders |
| `pressed` | `#c7d2dc` | `#33424f` | Card borders (as in the Figma), avatar fill |
| `accent` | `#0e96d1` | `#1b9ad3` | Primary buttons, slider, checked boxes, focus ring |
| `accent-ink` | `#0d89be` | `#5ec2ef` | Accent text: active navigation, links |
| `ok-bg` / `ok` | `#e7f6ec` / `#1b7443` | `#143326` / `#74d39f` | Satisfied requisites |
| `warn-bg` / `warn` | `#fef7e6` / `#8a5b00` | `#3a2f12` / `#eec35f` | Planned or in-progress requisites |
| `danger-bg` / `danger` | `#fdecee` / `#c4314b` | `#3c1c22` / `#ff8d9c` | Errors and destructive actions |
| `hover` / `active` | ink at 7% / 12% | ink at 9% / 15% | Hover and pressed fills that work on any surface |

Type is Inter, with utilities named after the Figma text styles:

| Class | Size | Use |
| --- | --- | --- |
| `text-heading-3` | 24/32 medium | Page titles |
| `text-heading-4` | 20/28 medium | Profile section titles, the desktop search field |
| `text-heading-5` | 18/26 medium | Rail titles, card titles, dialog titles |
| `text-heading-6` | 16/24 medium | Navigation, small headings |
| `text-body-1` | 16/24 | Body text |
| `text-body-2` | 14/20 | Secondary body text, fields |
| `text-label-lg` | 14/20 semibold | Buttons, rail section labels, checkboxes |
| `text-label-md` | 12/16 medium | Chips, small buttons, counts |
| `text-label-sm` | 10/14 | Phone tags |

Radii are `rounded-tag` (4px: chips, compact selects, stat boxes), `rounded-field` (10px: fields) and `rounded-card`
(10px); buttons and pills are `rounded-full`. Shadows are `shadow-header`, `shadow-popover` and `shadow-dialog`. Use
`tabular` for numbers that line up.

Dark mode is the `dark` class on `<html>`. An inline script in `app.html` sets it before the first paint from the saved
choice (`cmucourses:theme` in localStorage) or the system setting, and `theme` in
`lib/components/shell/theme.svelte.ts` keeps it in sync afterwards. `.light` and `.dark` also work on any element, so a
subtree can show the other theme.

## Conventions

- Icons are `@lucide/svelte`, sized with classes (`size-5` in buttons, `size-6` in navigation). Decorative icons get
  `aria-hidden="true"`; icon-only buttons use `IconButton`, which requires a `label`.
- Focus shows the global 2px accent ring. Do not remove outlines without a visible replacement.
- Source files are ASCII: write symbols as icons or HTML entities.
- Device conveniences (theme, cards or list, a hidden rail) go through `readLocal`/`writeLocal` in
  `lib/components/ui/storage.ts`, which survive blocked storage. Personal data never goes to localStorage
  ([decision 17](decisions/0017-store-personal-data-on-the-server.md)).

## Components

Import primitives from `$lib/components/ui`, layouts from `$lib/components/layout` and shell pieces from
`$lib/components/shell`. All take a `class` prop for layout tweaks.

| Component | Key props | Notes |
| --- | --- | --- |
| `Button` | `variant` primary, secondary, outline (default), ghost, danger; `size` sm, md, lg; `icon`, `trailingIcon`, `href`, `loading`, `full` | Pill buttons from the Figma; `href` renders a link, external links open a new tab |
| `IconButton` | `label` (required), `icon`, `size`, `variant` ghost, outline, solid; `pressed`, `href` | |
| `TextInput` | `bind:value`, `label`, `hideLabel`, `hint`, `error`, `icon`, `size` md, sm | |
| `SearchInput` | `bind:value`, `label`, `placeholder`, `variant` underline (page search), field; `onsubmit` | Escape clears; has a clear button |
| `Select` | `bind:value`, `options`, `placeholder`, `label`, `variant` field (rail), compact (toolbar) | Styled native select |
| `Combobox` | `bind:value` (string or null), `options`, `label`, `placeholder`, `clearable` | Type to filter; for departments |
| `MultiSelect` | `bind:value` (string array), `options`, `label`, `placeholder` | Searchable checklist; for requirements |
| `Checkbox` | `bind:checked`, `indeterminate`, `label`, `description` | |
| `Switch` | `bind:checked`, `label`, `hideLabel`, `icon`, `tone` accent, neutral | |
| `RangeSlider` | `bind:low`, `bind:high`, `min`, `max`, `step`, `label`, `presets` | Two thumbs; presets are the unit pills |
| `ToggleChips` | `bind:value` (array), `options`, `multiple`, `label` | Pressable pills, such as days |
| `Chip` | `tone` outline, soft, neutral, ok, warn, accent, strong; `size` xs, sm, md; `icon`, `href`, `onremove`, `onclick` | `onremove` makes a removable filter chip |
| `StatusChip` | `status` satisfied, planned, in-progress, unsatisfied | Check, arrow or x on the status colors |
| `CourseChip` | `code`, `title`, `status`, `tone` | Links to `/course/[code]` |
| `Card` | `as`, `href`, `padding` none, sm, md, lg; `interactive` | White card with the Figma border |
| `Dialog` | `bind:open`, `title`, `description`, `size`, `dismissible`, `footer` snippet | Modal; traps focus, Escape, scroll lock, returns focus |
| `Sheet` | `bind:open`, `title`, `side` left, right, bottom; `backLabel` | Full screen on phones; content stays mounted |
| `Popover` | `bind:open`, `trigger` snippet (spread its props on a button), `placement`, `matchWidth` | Closes on outside click and Escape |
| `Menu` | `items` (label, icon, onselect or href, danger), `label`, `trigger` snippet | Arrow keys, Home, End |
| `Tooltip` | `text`, `trigger` snippet | Hover and focus |
| `toast()` | `toast(message, { kind, action, duration })` | Rendered by the root layout's `Toaster` |
| `Tabs` | `tabs`, `bind:value`, `label`, `panel` snippet | |
| `SegmentedControl` | `options` (label, icon, iconOnly), `bind:value`, `label` | Radio group |
| `ViewToggle` | `pageKey`, `bind:value` cards or list | Remembers the choice per page |
| `Avatar` | `name`, `src`, `size` xs, sm, md, lg; `tone` | Initials, or a silhouette without a name |
| `AvatarGroup` | `people`, `max`, `size`, `label` | "+N" after `max` |
| `StarRating` | `value`, `max`, `size`, `showValue`, `label` | Partial stars for FCE rates |
| `StatBox` | `value`, `unit`, `label`, `variant` outline, plain; children | The "15.19 hrs/wk Workload" boxes |
| `DataTable` | `columns`, `rows`, `caption`, `variant` well (FCE), outline (finals); `cell` and `title` snippets | Scrolls horizontally; plain tables can use the `data-table` classes |
| `Spinner`, `Skeleton`, `EmptyState` | `size`; `variant`, `lines`; `title`, `description`, `icon`, actions as children | |

## Layout patterns

The root layout renders the header, `<main>` and the footer around every page. Pages pick one of four patterns.

**Rail layout** (courses, instructors, schedules). A white rail on the left with a title and a Hide
button, and the page on the right: a white toolbar band, then content on the page background. Below 1024px the rail is
a sheet opened from the toolbar's Filter button, which shows the active filter count.

```svelte
<RailLayout storageKey="courses" filterCount={active} backLabel="Search">
  {#snippet rail()}
    <RailGroup>
      <RailSection title="Department" icon={GraduationCap} badge={departments.length}>
        <MultiSelect options={departmentOptions} bind:value={departments} label="Departments" />
      </RailSection>
      <Button variant="secondary" full onclick={reset}>Reset</Button>
    </RailGroup>
    <RailGroup title="Aggregate FCEs">...</RailGroup>
  {/snippet}
  {#snippet header()}
    <PageToolbar title="Search" hideTitle="desktop" count={results.length} onreset={reset}>
      {#snippet search()}<SearchInput bind:value={query} label="Search courses" />{/snippet}
      {#snippet chips()}<Chip label="12 Units" onremove={clearUnits} />{/snippet}
      {#snippet actions()}<ViewToggle pageKey="courses" bind:value={view} />{/snippet}
    </PageToolbar>
  {/snippet}
  <!-- results -->
</RailLayout>
```

`RailGroup`s stack with dividers between them; `RailSection` is a collapsible filter with an icon, label and chevron.
`PageToolbar` has slots for the title, an intro, the search field, the result count, removable chips (plus a Reset
pill on phones) and right-aligned actions.

**Profile layout** (`/profile/[[section]]`). The profile rail: avatar, name, department and the section list (Saved
Courses, Currently Taking, Courses Taken, Schedules, Friends, Settings) with the active row filled. The section title,
`actions` and `filters` sit in a white band, or on the page background with `band={false}` as on Friends. On phones
the sections become an accordion with the open one's content under it.

**Plan layout** (`/plan`, [decision 36](decisions/0036-lay-out-the-degree-plan-beside-its-requirements.md)). A
full-width page: a header, then the semester grid beside a sticky requirements column from 1180px up, and tabs below
that.

**Page container** (course page, shared schedule, invite, errors). A centered column, `narrow`, `default` or `wide`.

Every list of courses or instructors offers cards and a one-column list. Put a `ViewToggle` with the page's key in the
toolbar actions and create the page's state with `storedView(pageKey) ?? "cards"` so the first render already uses the
device's choice.

## App shell

The header shows the logo, the navigation (Courses, Instructors, Schedules, Plan, Profile), the theme switch
and a Sign in or Sign out pill; below 1024px it becomes a menu button, the logo and the account control. The shell
reads the session from context:

```ts
interface ShellSession {
  readonly status: "loading" | "signed-out" | "signed-in";
  readonly name?: string;
  readonly detail?: string; // second line on the profile, such as the department
  signIn: () => void;
  signOut: () => void;
}
```

The root layout calls `setSession(...)`, currently with `createStubSession()` (always signed out); the data layer
replaces that with the real session, backed by runes state. Pages read it with `getSession()`, and `SignInPrompt`
shows the sign-in state for pages that need an account. Set each page's title with `<Title value="Instructors" />`.
