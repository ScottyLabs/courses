<!--
  The semester's schedules as tabs (Courseweave's ScheduleTabs), each with its units: switch, create (blank or from
  a degree plan's semester), rename (double-click or the menu), duplicate, share and delete. A tab synced with the
  active degree plan's semester is marked, as is one friends can see.
-->
<script lang="ts">
  import {
    CirclePlus,
    Copy,
    Ellipsis,
    GraduationCap,
    Link2,
    Pencil,
    PencilLine,
    Plus,
    RefreshCw,
    Trash2,
    Users,
  } from "@lucide/svelte";
  import { tick } from "svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Menu, { type MenuEntry } from "$lib/components/ui/Menu.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import type { Term } from "$lib/core/data";
  import { effectiveTermCourses } from "$lib/core/plan";
  import { plural, scheduleUnits } from "$lib/core/planner/format";
  import { entriesFromPlan } from "$lib/core/planner/fromPlan";
  import { codeOf } from "$lib/core/requisite-format";
  import { nextScheduleName } from "$lib/core/state/store";
  import { plans, schedules, type ScheduleItem } from "$lib/state";

  let {
    term,
    activeId,
    onshare,
    ontarget,
  }: {
    term: Term;
    /** The schedule shown, or null when the semester has none yet. */
    activeId: string | null;
    /** Opens sharing for the shown schedule. */
    onshare: () => void;
    /** The shown schedule's id, created when the semester has none yet. */
    ontarget: () => string;
  } = $props();

  const termId = $derived(term.meta.id);
  let tablist = $state<HTMLElement>();
  const list = $derived(schedules.forTerm(termId));
  const active = $derived(list.find((s) => s.id === activeId) ?? null);
  const placeholder = $derived(nextScheduleName(Object.values(schedules.items).filter((s) => s.termId === termId)));

  // Schedules a semester of the active degree plan is synced with.
  const activePlan = $derived(plans.active);
  const synced = $derived(
    new Set(
      activePlan?.terms
        .filter((t) => t.term === termId && t.link !== undefined && t.link !== null)
        .map((t) => t.link?.scheduleId ?? "") ?? [],
    ),
  );

  // Degree plans with courses in this semester can start a schedule.
  const fromPlans = $derived(
    plans.list
      .map((plan) => ({ plan, courses: effectiveTermCourses(plan, schedules.items).get(termId)?.courses ?? [] }))
      .filter((p) => p.courses.length > 0),
  );

  let renaming = $state<string | null>(null);
  let renameValue = $state("");
  let pendingDelete = $state<ScheduleItem | null>(null);
  let deleteOpen = $state(false);
  let tabs = $state<HTMLElement>();

  function create(): void {
    schedules.createSchedule(termId);
  }

  function createFromPlan(name: string, courses: { courseId: string; passFail?: boolean }[]): void {
    const { entries, skipped } = entriesFromPlan(courses, term);
    schedules.createSchedule(termId, `From ${name}`, entries);
    toast(
      skipped.length > 0
        ? `Added ${plural(entries.length, "course")} from ${name}. Not offered in ${term.meta.name}: ${skipped.map((id) => codeOf(id)).join(", ")}.`
        : `Added ${plural(entries.length, "course")} from ${name}.`,
      { kind: "success" },
    );
  }

  async function startRename(id: string): Promise<void> {
    const s = schedules.get(id);
    renameValue = s?.name ?? placeholder;
    renaming = id;
    await tick();
    const input = tabs?.querySelector<HTMLInputElement>("[data-rename]");
    input?.focus();
    input?.select();
  }

  function finishRename(save: boolean): void {
    const id = renaming;
    renaming = null;
    const name = renameValue.trim();
    if (id === null || !save || name === "") return;
    schedules.renameSchedule(id, name.slice(0, 120));
  }

  // Arrow keys move between tabs, as in the design system's Tabs.
  function onTabKey(event: KeyboardEvent): void {
    const i = list.findIndex((s) => s.id === activeId);
    let next = i;
    if (event.key === "ArrowRight") next = i + 1;
    else if (event.key === "ArrowLeft") next = i - 1;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = list.length - 1;
    else return;
    event.preventDefault();
    const target = list[(next + list.length) % list.length];
    if (target === undefined) return;
    schedules.setActiveSchedule(termId, target.id);
    void tick().then(() => tabs?.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]')?.focus());
  }

  const newItems = $derived<MenuEntry[]>([
    { label: "Blank schedule", icon: Plus, onselect: create },
    ...fromPlans.map(({ plan, courses }) => ({
      label: `From ${plan.name} (${plural(courses.length, "course")})`,
      icon: GraduationCap,
      onselect: () => createFromPlan(plan.name, courses),
    })),
  ]);

  const optionItems = $derived<MenuEntry[]>([
    { label: "Rename", icon: Pencil, onselect: () => void startRename(ontarget()) },
    {
      label: "Duplicate",
      icon: Copy,
      disabled: active === null,
      onselect: () => {
        if (active !== null) schedules.duplicateSchedule(active.id);
      },
    },
    { label: "Share", icon: Link2, onselect: onshare },
    "separator",
    {
      label: "Delete",
      icon: Trash2,
      danger: true,
      disabled: active === null,
      onselect: () => {
        pendingDelete = active;
        deleteOpen = true;
      },
    },
  ]);

  // Narrow screens scroll the tabs: keep the shown schedule's tab in view, such as a new one at the end.
  $effect(() => {
    void activeId;
    void list.length;
    const strip = tablist;
    const tab = strip?.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]');
    if (strip === undefined || tab === null || tab === undefined) return;
    const left = tab.offsetLeft - strip.offsetLeft;
    if (left < strip.scrollLeft) strip.scrollLeft = left;
    else if (left + tab.offsetWidth > strip.scrollLeft + strip.clientWidth)
      strip.scrollLeft = left + tab.offsetWidth - strip.clientWidth;
  });
</script>

<div bind:this={tabs} class="flex items-end gap-2 border-b border-line">
  <div
    bind:this={tablist}
    role="tablist"
    aria-label="Schedules"
    class="scrollbar-thin flex min-w-0 items-end gap-5 overflow-x-auto"
  >
    {#each list.length > 0 ? list : [null] as s (s?.id ?? "placeholder")}
      {@const id = s?.id ?? null}
      {@const isActive = id === activeId || s === null}
      {#if renaming !== null && (renaming === id || (s === null && renaming === activeId))}
        <input
          data-rename
          bind:value={renameValue}
          aria-label="Schedule name"
          maxlength={120}
          onblur={() => finishRename(true)}
          onkeydown={(event) => {
            if (event.key === "Enter") finishRename(true);
            if (event.key === "Escape") {
              event.preventDefault();
              finishRename(false);
            }
          }}
          class="mb-1.5 h-9 w-52 shrink-0 rounded-field border border-accent bg-surface px-3 text-heading-6 text-strong outline-none"
        />
      {:else}
        <button
          type="button"
          role="tab"
          aria-selected={isActive}
          tabindex={isActive ? 0 : -1}
          onclick={() => {
            if (id !== null) schedules.setActiveSchedule(termId, id);
          }}
          onkeydown={onTabKey}
          ondblclick={() => void startRename(id ?? ontarget())}
          title={isActive ? "Double-click to rename" : undefined}
          class="relative -mb-px flex shrink-0 items-center gap-2 border-b-2 pt-1 pb-3 transition-colors {isActive
            ? 'border-accent text-strong'
            : 'border-transparent text-muted hover:text-ink'}"
        >
          <span class="max-w-52 truncate {isActive ? 'text-heading-4' : 'text-heading-6'}">{s?.name ?? placeholder}</span>
          {#if s !== null && synced.has(s.id)}
            <span title="Synced with {activePlan?.name}" class="inline-flex text-ok">
              <RefreshCw aria-hidden="true" class="size-4" />
              <span class="sr-only">(synced with {activePlan?.name})</span>
            </span>
          {/if}
          {#if s?.visibility === "friends"}
            <span title="Friends can see it" class="inline-flex text-muted">
              <Users aria-hidden="true" class="size-4" />
              <span class="sr-only">(friends can see it)</span>
            </span>
          {/if}
          <span class="tabular text-label-md font-normal text-muted"
            >{plural(s === null ? 0 : scheduleUnits(s, term), "unit")}</span
          >
        </button>
      {/if}
    {/each}
  </div>

  <div class="mb-1.5 flex shrink-0 items-center">
    <IconButton label="Rename schedule" icon={PencilLine} size="sm" onclick={() => void startRename(ontarget())} />
    {#if fromPlans.length > 0}
      <Menu items={newItems} label="New schedule">
        {#snippet trigger(props)}
          <IconButton {...props} label="New schedule" icon={CirclePlus} size="sm" />
        {/snippet}
      </Menu>
    {:else}
      <IconButton label="New schedule" icon={CirclePlus} size="sm" onclick={create} />
    {/if}
    <Menu items={optionItems} label="Options for {active?.name ?? placeholder}">
      {#snippet trigger(props)}
        <IconButton {...props} label="Options for {active?.name ?? placeholder}" icon={Ellipsis} size="sm" />
      {/snippet}
    </Menu>
  </div>
</div>

<Dialog bind:open={deleteOpen} title="Delete this schedule?" size="sm">
  {#if pendingDelete}
    <p class="text-body-2 text-ink">
      &ldquo;{pendingDelete.name}&rdquo; and its {plural(pendingDelete.entries.length, "course")} will be removed. This
      can't be undone.
    </p>
  {/if}
  {#snippet footer()}
    <Button onclick={() => (deleteOpen = false)}>Cancel</Button>
    <Button
      variant="danger"
      onclick={() => {
        if (pendingDelete !== null) schedules.deleteSchedule(pendingDelete.id);
        pendingDelete = null;
        deleteOpen = false;
      }}>Delete schedule</Button
    >
  {/snippet}
</Dialog>
