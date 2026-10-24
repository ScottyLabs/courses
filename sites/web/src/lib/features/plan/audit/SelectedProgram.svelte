<script lang="ts">
  // A chosen program: its name and kind, the requirements version (entry year, campus), optional concentrations,
  // and what's checked alongside it.
  import { X } from "@lucide/svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { ProgramVersion } from "$lib/core/degree-types";
  import type { ProgramInfo, ProgramSelection } from "$lib/core/plan-types";
  import { KIND_LABEL, programCampus } from "./programs";

  let {
    selection,
    info,
    version,
    onchange,
    onremove,
  }: {
    selection: ProgramSelection;
    info: ProgramInfo | undefined;
    version: ProgramVersion | undefined;
    onchange: (next: ProgramSelection) => void;
    onremove: () => void;
  } = $props();

  const name = $derived(info?.name ?? version?.programName ?? `Program ${selection.catalogId}`);
  const campus = $derived(info ? programCampus(info) : null);
  const chosen = $derived(new Set(selection.concentrations ?? []));
  const attached = $derived(version?.attached.filter((a) => a.auditId !== null) ?? []);
  const versionOptions = $derived(
    (info?.versions ?? []).map((v) => ({
      value: String(v.auditId),
      label: `${v.name}${v.campus !== null && !v.name.includes(v.campus) ? ` (${v.campus})` : ""}`,
    })),
  );

  function toggleConcentration(id: number): void {
    const next = new Set(chosen);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    onchange({ ...selection, concentrations: [...next] });
  }
</script>

<li class="rounded-card border border-pressed bg-surface p-3">
  <div class="flex items-start gap-2">
    <div class="min-w-0 flex-1">
      <div class="flex flex-wrap items-center gap-x-2 gap-y-1">
        <span class="text-label-lg text-strong">{name}</span>
        {#if campus}<Chip tone="neutral" size="sm" label={campus} />{/if}
        {#if info}<Chip tone="soft" size="sm" label={KIND_LABEL[info.kind]} />{/if}
      </div>
      {#if info && info.versions.length > 1}
        <Select
          class="mt-2"
          variant="compact"
          label="Requirements version for {name}"
          value={String(selection.auditId)}
          options={versionOptions}
          onchange={(event) =>
            onchange({ catalogId: selection.catalogId, auditId: Number(event.currentTarget.value) })}
        />
      {:else if info?.versions[0]}
        <p class="mt-1 text-label-md font-normal text-muted">{info.versions[0].name} requirements</p>
      {/if}
    </div>
    <IconButton label="Remove {name}" icon={X} size="sm" class="max-md:size-10" onclick={onremove} />
  </div>

  {#if version && version.concentrations.length > 0}
    <div class="mt-2">
      <p class="text-label-md font-normal text-muted">Concentration (optional)</p>
      <div class="mt-1 flex flex-wrap gap-1.5">
        {#each version.concentrations as c (c.id)}
          <Chip
            shape="pill"
            pressed={chosen.has(c.id)}
            onclick={() => toggleConcentration(c.id)}
            title={c.programs[0] ? `Audited with ${c.programs[0].name}` : "Requirements for this concentration aren't available"}
            class="max-md:h-10"
            label={c.name}
          />
        {/each}
      </div>
    </div>
  {/if}

  {#if attached.length > 0}
    <p class="mt-2 text-label-md font-normal text-muted">Also checks {attached.map((a) => a.name).join(", ")}</p>
  {/if}
</li>
