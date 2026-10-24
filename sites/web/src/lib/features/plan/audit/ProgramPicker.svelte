<script lang="ts">
  // Choose majors, additional majors, minors and concentrations to audit, each at a requirements version (catalog
  // year). Degree programs are member data: signed out, this asks to sign in. Courseweave's ProgramPicker.
  import { Plus } from "@lucide/svelte";
  import { catalog, resource } from "$lib/data";
  import { defaultVersion } from "$lib/core/programs";
  import type { ProgramInfo, ProgramSelection } from "$lib/core/plan-types";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import SearchInput from "$lib/components/ui/SearchInput.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { countKind, KIND_FILTERS, KIND_LABEL, programCampus, searchPrograms, type KindFilter } from "./programs";
  import SelectedProgram from "./SelectedProgram.svelte";

  let {
    value,
    onchange,
    entryYearHint,
  }: {
    value: ProgramSelection[];
    onchange: (value: ProgramSelection[]) => void;
    /** Entry year (the fall the plan starts), to preselect matching requirement versions. */
    entryYearHint?: number;
  } = $props();

  const LIMIT = 30;
  const listId = $props.id();
  let query = $state("");
  let filter = $state<KindFilter>("all");

  const index = resource(() => catalog.programsIndex());
  const idsKey = $derived(value.map((v) => v.auditId).join(","));
  const versions = resource(() =>
    idsKey === "" ? null : catalog.programs(idsKey.split(",").map(Number)),
  );

  const programs = $derived(index.value?.programs ?? []);
  const byCatalog = $derived(new Map(programs.map((p) => [p.catalogId, p])));
  const results = $derived(
    searchPrograms(programs, query, filter, new Set(value.map((v) => v.catalogId))),
  );

  function add(p: ProgramInfo): void {
    const v = defaultVersion(p, entryYearHint);
    if (v === null) return;
    onchange([...value, { catalogId: p.catalogId, auditId: v.auditId }]);
    query = "";
  }
</script>

{#if index.status === "signin" || index.status === "forbidden"}
  <SignInPrompt
    title={index.status === "signin" ? "Sign in to pick your programs" : "Degree programs need a CMU account"}
    description="Majors, minors and their requirements come from CMU's degree audit, so they're only for signed-in CMU students. Your semesters still work without them."
  />
{:else if index.status === "error"}
  <p role="alert" class="text-body-2 text-danger">Couldn't load degree programs. {index.error} Reload to try again.</p>
{:else}
  <div class="flex flex-col gap-3">
    {#if value.length > 0}
      <ul class="flex flex-col gap-2">
        {#each value as selection, i (`${selection.catalogId}-${i}`)}
          <SelectedProgram
            {selection}
            info={byCatalog.get(selection.catalogId)}
            version={versions.value?.get(selection.auditId)}
            onchange={(next) => onchange(value.map((v, j) => (j === i ? next : v)))}
            onremove={() => onchange(value.filter((_, j) => j !== i))}
          />
        {/each}
      </ul>
    {/if}

    <div class="flex flex-col gap-2">
      <SearchInput
        bind:value={query}
        variant="field"
        label="Search degree programs"
        placeholder={value.length > 0 ? "Add a minor, major or concentration" : "Search majors and minors"}
        aria-controls={listId}
      />
      <div class="flex flex-wrap gap-1.5" role="group" aria-label="Program type">
        {#each KIND_FILTERS as f (f.value)}
          <Chip
            shape="pill"
            pressed={filter === f.value}
            onclick={() => (filter = f.value)}
            class="max-md:h-10"
            label={f.label}
          />
        {/each}
      </div>
      {#if index.status === "loading" && index.value === undefined}
        <p class="flex items-center gap-2 text-body-2 text-muted"><Spinner size="sm" decorative /> Loading programs</p>
      {:else if query.trim() !== ""}
        {#if results.length > 0}
          <ul id={listId} class="scrollbar-thin flex max-h-80 flex-col overflow-y-auto rounded-field border border-line">
            {#each results.slice(0, LIMIT) as p (p.catalogId)}
              {@const campus = programCampus(p)}
              <li class="border-t border-line first:border-t-0">
                <button
                  type="button"
                  onclick={() => add(p)}
                  class="flex min-h-11 w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-paper md:min-h-9"
                >
                  <Plus aria-hidden="true" class="size-4 shrink-0 text-muted" />
                  <span class="min-w-0 flex-1 text-body-2 text-strong">{p.name}</span>
                  {#if campus}<Chip tone="neutral" size="sm" label={campus} />{/if}
                  <Chip tone="soft" size="sm" label={KIND_LABEL[p.kind]} />
                </button>
              </li>
            {/each}
            {#if results.length > LIMIT}
              <li class="border-t border-line px-3 py-2 text-label-md font-normal text-muted">
                {results.length - LIMIT} more; keep typing to narrow it down
              </li>
            {/if}
          </ul>
        {:else}
          <p class="text-body-2 text-muted">
            No programs match "{query.trim()}". Try fewer words, or another program type.
          </p>
        {/if}
      {:else if value.length === 0 && programs.length > 0}
        <p class="text-label-md font-normal text-muted">
          {countKind(programs, "major")} majors, {countKind(programs, "minor")} minors and {countKind(
            programs,
            "additional-major",
          )} additional majors, from CMU's degree audit.
        </p>
      {/if}
    </div>
  </div>
{/if}
