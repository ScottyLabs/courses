<!--
  "Send to Stellic or SIO" (Courseweave's SendToStellic): pick schedules (one a semester) or a degree plan, then copy a
  script the student runs in their own signed-in Stellic (or SIO) tab. The script shows what it will add and changes
  nothing until the student confirms there; it only adds courses with their sections, never registers, drops or
  removes anything. For SIO there is also the plain list of course numbers and sections. Nothing here talks to
  Stellic or SIO itself. The schedules page opens it with mode "schedules", the degree plan with mode "plan".
-->
<script lang="ts">
  import { ExternalLink } from "@lucide/svelte";
  import CopyField from "$lib/components/ui/CopyField.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import { catalog, resource } from "$lib/data";
  import { codeOf } from "$lib/core/requisite-format";
  import { currentTermCode, resolvePlan } from "$lib/core/plan";
  import { termOrdinal } from "$lib/core/requisites";
  import {
    SIO_PLAN_URL,
    STELLIC_ORIGIN,
    buildSioScript,
    buildStellicScript,
    exportPlan,
    exportSchedule,
    sioText,
    stellicTerm,
    type ExportTerm,
  } from "$lib/core/stellic-export";
  import { plans, schedules, type ScheduleItem } from "$lib/state";

  let {
    open = $bindable(false),
    mode = "schedules",
    initialId,
    courseTitle,
  }: {
    open?: boolean;
    /** schedules: one schedule a semester; plan: a degree plan's current and future semesters. */
    mode?: "schedules" | "plan";
    /** The schedule (or plan) to start with. */
    initialId?: string;
    /** Course titles for the script; defaults to the catalog's. The plan passes its own, which know special listings. */
    courseTitle?: (courseId: string) => string;
  } = $props();

  const index = resource(() => catalog.catalogIndex());
  const titleOf = $derived(
    courseTitle ?? ((id: string): string => index.value?.courses[id]?.title ?? codeOf(id)),
  );

  // Schedules with courses, newest semester first; at most one a semester (two would ask for conflicting sections).
  const byTerm = $derived.by(() => {
    const groups = new Map<string, ScheduleItem[]>();
    for (const s of Object.values(schedules.items))
      if (s.entries.length > 0) groups.set(s.termId, [...(groups.get(s.termId) ?? []), s]);
    return [...groups].toSorted(([a], [b]) => (termOrdinal(b) ?? 0) - (termOrdinal(a) ?? 0));
  });

  let chosen = $state<Record<string, string>>({});
  let planId = $state("");

  $effect(() => {
    if (!open) return;
    if (mode === "plan") planId = initialId ?? plans.activeId ?? plans.list[0]?.id ?? "";
    else {
      const first = initialId === undefined ? undefined : schedules.get(initialId);
      chosen = first === undefined ? {} : { [first.termId]: first.id };
    }
  });

  const terms = $derived.by((): ExportTerm[] => {
    if (mode === "plan") {
      const plan = plans.get(planId);
      return plan === undefined
        ? []
        : exportPlan(resolvePlan(plan, schedules.items), schedules.items, currentTermCode(), titleOf);
    }
    return Object.values(chosen)
      .map((id) => schedules.get(id))
      .filter((s): s is ScheduleItem => s !== undefined)
      .map((s) => exportSchedule(s, titleOf))
      .filter((t): t is ExportTerm => t !== null && t.courses.length > 0)
      .toSorted((a, b) => (termOrdinal(a.term) ?? 0) - (termOrdinal(b.term) ?? 0));
  });
  const courseCount = $derived(terms.reduce((n, t) => n + t.courses.length, 0));
  const script = $derived(terms.length > 0 ? buildStellicScript(terms) : "");
  const sioScript = $derived(terms.length > 0 ? buildSioScript(terms) : "");

  function pick(termId: string, id: string, on: boolean): void {
    const next = { ...chosen };
    if (on) next[termId] = id;
    else delete next[termId];
    chosen = next;
  }

  function summary(s: ScheduleItem): string {
    return s.entries
      .filter((e) => !e.hidden)
      .map((e) => `${codeOf(e.courseId)}${e.child ?? e.section ? ` ${e.child ?? e.section}` : ""}`)
      .join(", ");
  }
</script>

<Dialog bind:open title={mode === "plan" ? "Send plan to Stellic" : "Send to Stellic or SIO"} size="lg">
  {#if open}
    <div class="flex flex-col gap-6">
      {#if mode === "schedules"}
        <section aria-labelledby="send-pick" class="flex flex-col gap-2">
          <h3 id="send-pick" class="text-heading-6 text-strong">Schedules</h3>
          <p class="text-body-2 text-muted">One a semester.</p>
          {#if byTerm.length === 0}
            <p class="text-body-2 text-ink">Add courses to a schedule first.</p>
          {:else}
            {#each byTerm as [termId, list] (termId)}
              <fieldset class="rounded-field border border-line p-2.5">
                <legend class="px-1 text-label-md text-ink">{stellicTerm(termId)?.name ?? termId}</legend>
                {#each list as s (s.id)}
                  <Checkbox
                    checked={chosen[termId] === s.id}
                    label={s.name}
                    description={summary(s)}
                    onchange={(event) => pick(termId, s.id, (event.currentTarget as HTMLInputElement).checked)}
                    class="flex w-full"
                  />
                {/each}
              </fieldset>
            {/each}
          {/if}
        </section>
      {:else}
        <section class="flex flex-col gap-2">
          <Select
            bind:value={planId}
            label="Degree plan"
            hideLabel={false}
            options={plans.list.map((p) => ({ value: p.id, label: p.name }))}
          />
          <p class="text-body-2 text-muted">
            Sends this plan's current and future semesters; past ones are already in Stellic from your transcript.
            Semesters synced with a schedule include its sections. Stellic keeps one main plan, so send one plan at a
            time.
          </p>
          {#if terms.length > 0}
            <ul class="flex flex-col gap-0.5 text-body-2 text-ink">
              {#each terms as t (t.term)}
                <li><span class="font-semibold text-strong">{t.name}</span>: {t.courses.map((c) => c.code).join(", ")}</li>
              {/each}
            </ul>
          {:else if planId !== ""}
            <p class="text-body-2 text-ink">This plan has no courses in this or later semesters yet.</p>
          {/if}
        </section>
      {/if}

      <section aria-labelledby="send-stellic" class="flex flex-col gap-3 border-t border-line pt-5">
        <h3 id="send-stellic" class="text-heading-5 text-strong">Stellic</h3>
        <p class="text-body-2 text-ink">
          Stellic has no import, so this is a short script you run in your own signed-in Stellic tab. It shows you
          exactly what it will add and changes nothing until you confirm. It only adds courses (with your sections) to
          your plan: it never registers, drops or removes anything.
        </p>
        <ol class="flex list-decimal flex-col gap-2 pl-5 text-body-2 text-ink">
          <li>
            Copy the script{courseCount > 0 ? ` (${courseCount} ${courseCount === 1 ? "course" : "courses"})` : ""}:
            {#if script}
              <div class="mt-1.5">
                <CopyField value={script} label="Stellic script" rows={4} copyLabel="Copy script" />
              </div>
            {:else}
              <span class="text-muted">choose {mode === "plan" ? "a plan with courses" : "a schedule"} first.</span>
            {/if}
          </li>
          <li>
            Open <a href="{STELLIC_ORIGIN}/" target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1 font-semibold text-accent-ink underline">Stellic<ExternalLink aria-hidden="true" class="size-3.5" /><span class="sr-only">(opens in a new tab)</span></a>
            and sign in.
          </li>
          <li>
            Open the browser console: <kbd class="rounded-tag border border-line px-1 text-label-md">&#8997;&#8984;J</kbd> on
            a Mac or <kbd class="rounded-tag border border-line px-1 text-label-md">Ctrl+Shift+J</kbd> in Chrome or Edge.
            In Safari, turn on Show features for web developers in Settings &rarr; Advanced, then
            <kbd class="rounded-tag border border-line px-1 text-label-md">&#8997;&#8984;C</kbd>.
          </li>
          <li>Paste the script and press Enter. Chrome may first ask you to type &ldquo;allow pasting&rdquo;.</li>
          <li>Check the list it shows and confirm. Reload Stellic to see the courses.</li>
        </ol>
        <p class="text-label-md font-normal text-muted">
          Only paste code into a console from a source you trust. You can read this one in the box above first.
        </p>
      </section>

      {#if mode === "schedules"}
        <section aria-labelledby="send-sio" class="flex flex-col gap-3 border-t border-line pt-5">
          <h3 id="send-sio" class="text-heading-5 text-strong">SIO planned schedule</h3>
          <p class="text-body-2 text-ink">
            A script for SIO's Plan Course Schedule page: it adds the courses to the planned schedule you're viewing with
            SIO's own Add box and section buttons, shows you the list first, and changes nothing until you confirm.
            Planned schedules are SIO's planning tool; nothing is registered.
          </p>
          <ol class="flex list-decimal flex-col gap-2 pl-5 text-body-2 text-ink">
            <li>
              Copy the SIO script:
              {#if sioScript}
                <div class="mt-1.5"><CopyField value={sioScript} label="SIO script" rows={4} copyLabel="Copy script" /></div>
              {:else}
                <span class="text-muted">choose a schedule first.</span>
              {/if}
            </li>
            <li>
              Open SIO's <a href={SIO_PLAN_URL} target="_blank" rel="noopener noreferrer" class="inline-flex items-center gap-1 font-semibold text-accent-ink underline">Plan Course Schedule<ExternalLink aria-hidden="true" class="size-3.5" /><span class="sr-only">(opens in a new tab)</span></a>
              and choose the planned schedule to fill, or make one with Plan New Schedule.
            </li>
            <li>Open the console, paste the script and press Enter, then confirm the list.</li>
            {#if terms.length > 1}
              <li>It fills the semester you're viewing. For another semester, switch to a plan for it and run it again.</li>
            {/if}
          </ol>
          {#if terms.length > 0}
            <p class="text-body-2 text-ink">Or enter them yourself, by course number and the section to register for:</p>
            <div class="data-table-scroll rounded-field border border-line">
              <table class="w-full text-left text-body-2">
                <caption class="sr-only">Courses and sections to enter in SIO</caption>
                <thead class="border-b border-line bg-surface-2 text-label-md text-muted">
                  <tr>
                    <th scope="col" class="px-3 py-2 font-medium">Semester</th>
                    <th scope="col" class="px-3 py-2 font-medium">Course</th>
                    <th scope="col" class="px-3 py-2 font-medium">Section</th>
                    <th scope="col" class="px-3 py-2 font-medium">Title</th>
                  </tr>
                </thead>
                <tbody class="tabular">
                  {#each terms as t (t.term)}
                    {#each t.courses as c (c.code)}
                      <tr class="border-b border-line last:border-b-0">
                        <td class="px-3 py-2 whitespace-nowrap text-ink">{t.name}</td>
                        <td class="px-3 py-2 font-semibold text-strong">{c.code.replace("-", "")}</td>
                        <td class="px-3 py-2 text-strong">
                          {#if c.sections.at(-1)}{c.sections.at(-1)}{:else}<span class="text-warn">pick one</span>{/if}
                        </td>
                        <td class="max-w-64 truncate px-3 py-2 text-ink">{c.title}</td>
                      </tr>
                    {/each}
                  {/each}
                </tbody>
              </table>
            </div>
            <CopyField value={sioText(terms)} label="The list as text" rows={3} copyLabel="Copy list" />
          {/if}
        </section>
      {/if}
    </div>
  {/if}
</Dialog>
