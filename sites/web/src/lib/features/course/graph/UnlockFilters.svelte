<script lang="ts">
  // Filters for a long list of what a course unlocks: only courses that need it outright, only this semester's, only
  // the ones the student could take next, one department (with each department's count).
  import Chip from "$lib/components/ui/Chip.svelte";
  import Select from "$lib/components/ui/Select.svelte";

  let {
    requiredOnly = $bindable(),
    offeredOnly = $bindable(),
    readyOnly = $bindable(),
    department = $bindable(),
    termName,
    canReady,
    departments,
    shown,
    total,
  }: {
    requiredOnly: boolean;
    offeredOnly: boolean;
    readyOnly: boolean;
    /** A course-number prefix, or "" for every department. */
    department: string;
    /** The semester's name, for "Offered in"; omitted where there's no semester to check. */
    termName?: string;
    /** Some unlock is one the student could take next. */
    canReady: boolean;
    departments: readonly { value: string; label: string }[];
    shown: number;
    total: number;
  } = $props();

  const filtering = $derived(requiredOnly || offeredOnly || readyOnly || department !== "");
  const chip = "h-10 px-3 sm:h-8";
</script>

<div class="flex flex-wrap items-center gap-2 border-b border-line px-4 py-2.5">
  <span class="mr-1 text-label-md text-muted">Unlocks</span>
  <Chip
    shape="pill"
    size="md"
    pressed={requiredOnly}
    title="Only courses whose every prerequisite path needs this course"
    onclick={() => (requiredOnly = !requiredOnly)}
    class={chip}>Requires it outright</Chip
  >
  {#if termName !== undefined}
    <Chip shape="pill" size="md" pressed={offeredOnly} onclick={() => (offeredOnly = !offeredOnly)} class={chip}
      >Offered in {termName}</Chip
    >
  {/if}
  {#if canReady}
    <Chip
      shape="pill"
      size="md"
      pressed={readyOnly}
      title="Courses whose other prerequisites you've met"
      onclick={() => (readyOnly = !readyOnly)}
      class={chip}>You could take next</Chip
    >
  {/if}
  <Select label="Department" variant="compact" options={departments} bind:value={department} class="w-[16rem] max-w-full" />
  <span class="text-label-md text-muted tabular" aria-live="polite">{filtering ? `${shown} of ${total}` : ""}</span>
</div>
