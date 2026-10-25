<script lang="ts">
  // The import's review step: the groups the PDF produced (exam credit, transfer credit, each semester), each item
  // checkable, a Stellic plan's "create the degree plan" choice and profile updates. Courseweave's ReviewPanel,
  // ItemRow and DraftOption in features/record/SioImport.tsx.
  import Checkbox from "$lib/components/ui/Checkbox.svelte";
  import type { ImportItem } from "$lib/core/record/SioImportPlan";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import type { ImportReviewState } from "./SioImportButton.svelte";

  let {
    review,
    checked = $bindable(),
    useProfile = $bindable(),
    useDraft = $bindable(),
    profileText,
  }: {
    review: ImportReviewState;
    checked: Set<string>;
    useProfile: boolean;
    useDraft: boolean;
    /** "college to ... and first semester to ...", or "" when the PDF suggests no change. */
    profileText: string;
  } = $props();

  const STATUS_TEXT: Record<ImportItem["status"], string> = {
    graded: "",
    "in-progress": "In progress",
    "no-grade": "No grade",
    credit: "",
  };

  function toggle(key: string, on: boolean): void {
    const next = new Set(checked);
    if (on) next.add(key);
    else next.delete(key);
    checked = next;
  }

  function toggleAll(keys: string[], on: boolean): void {
    const next = new Set(checked);
    for (const k of keys) {
      if (on) next.add(k);
      else next.delete(k);
    }
    checked = next;
  }

  const draft = $derived(review.draft);
  const withCourses = $derived(draft?.terms.filter((t) => t.courses.length > 0) ?? []);
  const found = $derived(draft?.programNames.filter((p) => p.found).map((p) => p.name) ?? []);
  const missing = $derived(draft?.programNames.filter((p) => !p.found).map((p) => p.name) ?? []);
  const span = $derived.by(() => {
    const first = withCourses.at(0);
    const last = withCourses.at(-1);
    return first && last ? `${termLabel(first.term)} to ${termLabel(last.term)}` : null;
  });
</script>

<div class="flex flex-col gap-5">
  <p class="text-body-2 text-ink">{review.intro}</p>

  {#if draft}
    <div class="rounded-field border border-line px-3 py-2.5">
      <Checkbox bind:checked={useDraft} class="items-start">
        <span class="text-body-2">
          <span class="font-semibold text-strong">Create the degree plan &ldquo;{draft.name}&rdquo;</span>
          <span class="block font-normal text-ink">
            {draft.courseCount}
            {draft.courseCount === 1 ? "course" : "courses"}{span ? `, ${span}` : ""}{found.length > 0
              ? `. Programs: ${found.join(", ")}`
              : ""}.
          </span>
          {#if missing.length > 0}
            <span class="block font-normal text-muted">
              Not found here: {missing.join(", ")}. You can pick programs in the plan.
            </span>
          {/if}
        </span>
      </Checkbox>
      {#if withCourses.length > 0}
        <details class="mt-1 pl-9 text-body-2">
          <summary class="flex min-h-10 cursor-pointer items-center text-ink">Show semesters</summary>
          <dl class="mt-1 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 pb-1">
            {#each withCourses as t (t.term)}
              <dt class="text-muted">{termLabel(t.term)}</dt>
              <dd class="tabular min-w-0 text-ink">
                {t.courses.map((c) => codeOf(c.courseId)).join(", ")}
              </dd>
            {/each}
          </dl>
        </details>
      {/if}
    </div>
  {/if}

  {#if profileText !== ""}
    <div class="rounded-field border border-line px-3 py-1.5">
      <Checkbox bind:checked={useProfile} label="Set your {profileText}" class="font-normal" />
    </div>
  {/if}

  {#if review.plan.groups.length === 0 && !draft}
    <p class="rounded-field bg-surface-2 px-3 py-2.5 text-body-2 text-ink">
      {review.warnings.length > 0
        ? "No courses or credit could be read from this record."
        : "This record has no courses or credit yet."}
    </p>
  {/if}

  {#if draft && review.plan.groups.length > 0}
    <h3 class="-mb-3 text-label-lg text-strong">Add to your record</h3>
  {/if}

  {#each review.plan.groups as group (group.key)}
    {@const keys = group.items.map((i) => i.key)}
    {@const on = keys.filter((k) => checked.has(k)).length}
    <section aria-label={group.subtitle ? `${group.title}, ${group.subtitle}` : group.title}>
      <div class="flex flex-wrap items-center gap-x-3 border-b border-line pb-1">
        <Checkbox
          checked={on === keys.length}
          indeterminate={on > 0 && on < keys.length}
          onchange={(event) => toggleAll(keys, event.currentTarget.checked)}
        >
          <span class="text-heading-6 text-strong">{group.title}</span>
        </Checkbox>
        {#if group.subtitle}<span class="text-body-2 text-muted">{group.subtitle}</span>{/if}
        <span class="ml-auto text-label-md text-muted">{on} of {keys.length}</span>
      </div>
      <ul class="divide-y divide-line">
        {#each group.items as item (item.key)}
          {@const info = review.lookup(item.row.courseId)}
          {@const status = STATUS_TEXT[item.status]}
          {@const inputId = `import-${item.key.replaceAll(":", "-")}`}
          <li class="flex min-h-11 items-center gap-1">
            <Checkbox
              id={inputId}
              checked={checked.has(item.key)}
              onchange={(event) => toggle(item.key, event.currentTarget.checked)}
            />
            <label for={inputId} class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 py-1 sm:gap-3">
              <span class="tabular code-column text-label-lg text-strong">
                {codeOf(item.row.courseId)}
              </span>
              <span class="min-w-0 flex-1">
                <span class="block truncate text-body-2 text-ink">
                  {info.title ?? item.entry.label ?? item.row.title}
                </span>
                {#if item.already}
                  <span class="block text-label-md font-normal text-muted">Already in your record</span>
                {/if}
              </span>
              <span class="tabular shrink-0 text-body-2 text-muted">{item.row.units}u</span>
              <span class="shrink-0 text-right text-label-md sm:w-[76px]">
                {#if item.entry.grade}
                  <span class="rounded-tag bg-surface-2 px-1.5 py-0.5 text-strong">{item.entry.grade}</span>
                {:else if status}
                  <span class={item.status === "in-progress" ? "text-warn" : "text-muted"}>{status}</span>
                {/if}
              </span>
            </label>
          </li>
        {/each}
      </ul>
    </section>
  {/each}

  {#if review.warnings.length > 0}
    <details class="text-body-2 text-muted">
      <summary class="flex min-h-10 cursor-pointer items-center">
        {review.warnings.length}
        {review.warnings.length === 1 ? "line" : "lines"}
        {review.source === "stellic" ? "weren't imported" : "couldn't be read"}
      </summary>
      <ul class="mt-1 list-disc pl-5">
        {#each review.warnings as warning, index (index)}
          <li>{warning}</li>
        {/each}
      </ul>
    </details>
  {/if}
</div>
