<script lang="ts">
  // A planned course's options: move up or down, move to another semester, pass/fail, units for variable-unit
  // courses, waive or restore its requisites, view the course, remove. The keyboard (and touch) way to do what dragging
  // does.
  import { ArrowDown, ArrowLeftRight, ArrowUp, ExternalLink, ShieldCheck, Trash2, Undo2 } from "@lucide/svelte";
  import type { PlanCheck, PlanCourse } from "$lib/core/plan";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { record } from "$lib/state";
  import MenuItem from "./MenuItem.svelte";
  import MenuLabel from "./MenuLabel.svelte";
  import OptionsMenu from "./OptionsMenu.svelte";
  import type { CourseRowInfo } from "./rows";

  let {
    course,
    info,
    href,
    moveTargets,
    removeLabel = "Remove from plan",
    onmove,
    onmoveup,
    onmovedown,
    onremove,
    onupdate,
    check,
  }: {
    course: PlanCourse;
    info: CourseRowInfo;
    href: string;
    /** Other semesters it can move to. */
    moveTargets: string[];
    /** Synced semesters remove from their schedule. */
    removeLabel?: string;
    onmove: (term: string) => void;
    /** Within the semester; absent at the top or bottom. */
    onmoveup?: () => void;
    onmovedown?: () => void;
    onremove: () => void;
    onupdate: (patch: { passFail?: boolean; units?: number }) => void;
    /** The course's requisite check, for waiving or restoring its requisites. */
    check?: PlanCheck;
  } = $props();

  const canWaive = $derived(
    check?.check.issues.some((i) => i.severity !== "info" && (i.kind === "prereq" || i.kind === "coreq")) ?? false,
  );
  const waived = $derived((check?.check.waived.prereqs || check?.check.waived.coreqs) ?? false);

  const code = $derived(codeOf(course.courseId));

  function onunits(event: Event & { currentTarget: HTMLInputElement }): void {
    const text = event.currentTarget.value;
    const n = Number(text);
    onupdate({ units: text !== "" && Number.isFinite(n) ? n : undefined });
  }
</script>

<OptionsMenu label="Options for {code}">
  {#snippet children(close)}
    {#if onmoveup || onmovedown}
      <MenuItem icon={ArrowUp} disabled={!onmoveup} onclick={() => { close(); onmoveup?.(); }}>Move up</MenuItem>
      <MenuItem icon={ArrowDown} disabled={!onmovedown} onclick={() => { close(); onmovedown?.(); }}>Move down</MenuItem>
    {/if}
    {#if moveTargets.length > 0}
      <MenuLabel separated={onmoveup !== undefined || onmovedown !== undefined}>Move to</MenuLabel>
      <div class="scrollbar-thin max-h-52 overflow-y-auto">
        {#each moveTargets as t (t)}
          <MenuItem icon={ArrowLeftRight} onclick={() => { close(); onmove(t); }}>{termLabel(t)}</MenuItem>
        {/each}
      </div>
    {/if}
    <div role="separator" class="my-1.5 h-px bg-line"></div>
    <label
      class="flex min-h-10 items-center gap-3 px-3.5 text-body-2 md:min-h-9 {info.passFailOnly
        ? 'text-muted'
        : 'cursor-pointer text-ink hover:bg-paper'}"
    >
      <input
        type="checkbox"
        checked={info.passFail}
        disabled={info.passFailOnly}
        onchange={(event) => onupdate({ passFail: event.currentTarget.checked })}
        class="size-4 rounded-[3px] border-line-strong text-accent focus:ring-accent"
      />
      {info.passFailOnly ? "Pass/fail only" : "Take pass/fail"}
    </label>
    {#if info.variable}
      <label class="flex min-h-10 items-center gap-3 px-3.5 text-body-2 text-ink md:min-h-9">
        Units
        <input
          type="number"
          min="0"
          max="48"
          step="1"
          value={course.units ?? info.units}
          oninput={onunits}
          class="tabular h-8 w-20 rounded-tag border border-line-strong bg-surface px-2 text-body-2 text-ink"
        />
      </label>
    {/if}
    {#if canWaive}
      <MenuItem
        icon={ShieldCheck}
        onclick={() => {
          close();
          record.setOverride(course.courseId, { prereqs: true, coreqs: true, note: "Waived from the degree plan" });
        }}>Mark requisites as waived</MenuItem
      >
    {/if}
    {#if waived}
      <MenuItem icon={Undo2} onclick={() => { close(); record.setOverride(course.courseId, null); }}>Undo requisite waiver</MenuItem>
    {/if}
    <MenuItem icon={ExternalLink} {href} onclick={close}>View course</MenuItem>
    <MenuItem icon={Trash2} danger onclick={() => { close(); onremove(); }}>{removeLabel}</MenuItem>
  {/snippet}
</OptionsMenu>
