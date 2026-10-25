<script lang="ts">
  // A chosen program: its name and kind, the requirements version (entry year, campus), optional concentrations
  // (any number, as in Courseweave; press one again to clear it), and what's checked alongside it.
  import { X } from "@lucide/svelte";
  import Chip from "$lib/components/ui/Chip.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import ToggleChips from "$lib/components/ui/ToggleChips.svelte";
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
  const chosen = $derived(selection.concentrations ?? []);
  const concentrationOptions = $derived(
    (version?.concentrations ?? []).map((c) => ({ value: String(c.id), label: c.name })),
  );
  // What the audit checks alongside the program: its gen-ed bundles, then each chosen concentration's requirements.
  const chosenInfo = $derived(
    (version?.concentrations ?? []).filter((c) => chosen.includes(c.id)),
  );
  const alsoChecks = $derived([
    ...(version?.attached.filter((a) => a.auditId !== null).map((a) => a.name) ?? []),
    ...chosenInfo.flatMap((c) => (c.programs[0] ? [c.programs[0].name] : [])),
  ]);
  const unchecked = $derived(chosenInfo.filter((c) => c.programs.length === 0).map((c) => c.name));
  const versionOptions = $derived(
    (info?.versions ?? []).map((v) => ({
      value: String(v.auditId),
      label: `${v.name}${v.campus !== null && !v.name.includes(v.campus) ? ` (${v.campus})` : ""}`,
    })),
  );

  // The name's last word stays with the badges after it.
  const nameHead = $derived(name.includes(" ") ? name.slice(0, name.lastIndexOf(" ") + 1) : "");
  const nameTail = $derived(name.includes(" ") ? name.slice(name.lastIndexOf(" ") + 1) : name);
</script>

<li class="flex flex-col gap-2.5 py-3 first:pt-0">
  <div class="flex items-start gap-2">
    <!-- The badges follow the name's last word, so a long name doesn't leave them on a line of their own. -->
    <p class="min-h-8 min-w-0 flex-1 pt-1 text-label-lg text-strong">
      {nameHead}<span class="whitespace-nowrap"
        >{nameTail}{#if campus}<Chip tone="neutral" size="sm" label={campus} class="ml-2 align-[1px]" />{/if}{#if info}<Chip
            tone="soft"
            size="sm"
            label={KIND_LABEL[info.kind]}
            class="ml-2 align-[1px]"
          />{/if}</span
      >
    </p>
    <IconButton label="Remove {name}" icon={X} size="sm" class="-mr-1.5 max-md:size-10" onclick={onremove} />
  </div>

  {#if info && info.versions.length > 1}
    <Select
      class="w-full sm:w-72"
      label="Requirements version for {name}"
      value={String(selection.auditId)}
      options={versionOptions}
      onchange={(event) => onchange({ catalogId: selection.catalogId, auditId: Number(event.currentTarget.value) })}
    />
  {:else if info?.versions[0]}
    <p class="-mt-1.5 text-label-md font-normal text-muted">{info.versions[0].name} requirements</p>
  {/if}

  {#if concentrationOptions.length > 0}
    <div class="flex flex-col gap-1.5">
      <p class="text-label-md font-normal text-muted">Concentration (optional)</p>
      <ToggleChips
        label="Concentrations for {name}"
        check
        options={concentrationOptions}
        bind:value={
          () => chosen.map(String),
          (next) => onchange({ ...selection, concentrations: next.map(Number) })
        }
        class="max-md:[&>button]:h-10"
      />
    </div>
  {/if}

  {#if alsoChecks.length > 0 || unchecked.length > 0}
    <p class="text-label-md font-normal text-muted">
      {#if alsoChecks.length > 0}Also checks {alsoChecks.join(", ")}.{/if}
      {#if unchecked.length > 0}
        {unchecked.join(", ")}
        {unchecked.length === 1 ? "isn't" : "aren't"} checked: the degree audit doesn't publish {unchecked.length === 1
          ? "its"
          : "their"} requirements.
      {/if}
    </p>
  {/if}
</li>
