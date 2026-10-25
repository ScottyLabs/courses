<script lang="ts">
  // Courses Taken's opening panel while the record is empty: import, or jump to a section to add by hand.
  // Courseweave's RecordEmptyStart in features/record/RecordStart.tsx.
  import { toast } from "$lib/components/ui/toast.svelte";
  import HowTo from "./HowTo.svelte";
  import SioImportButton from "./SioImportButton.svelte";

  /** The record's sections, for "add by hand" shortcuts. */
  const SECTIONS = [
    { id: "record-taken", label: "Courses taken" },
    { id: "record-exams", label: "AP, IB and A-Level credit" },
    { id: "record-transfer", label: "Transfer credit" },
    { id: "record-waivers", label: "Placements and overrides" },
  ];

  function jump(id: string): void {
    const target = document.getElementById(id);
    target?.scrollIntoView({ behavior: "smooth", block: "start" });
    // Land keyboard and screen reader users in the section too.
    target?.closest("section")?.querySelector<HTMLElement>("input, select, button")?.focus({ preventScroll: true });
  }
</script>

<section aria-labelledby="record-start" class="overflow-hidden rounded-card border border-pressed bg-surface">
  <div class="flex flex-col gap-4 p-5 md:flex-row md:items-center md:gap-8 md:p-6">
    <div class="min-w-0 flex-1">
      <h2 id="record-start" class="text-heading-5 text-strong">Start with your SIO record</h2>
      <p class="mt-1 mb-2 max-w-[62ch] text-body-2 text-ink">
        Import your unofficial academic record, or your Stellic plan, to add your CMU courses, AP and IB credit and
        transfer credit in one go. You choose what to bring in.
      </p>
      <HowTo />
    </div>
    <SioImportButton variant="primary" onimported={(message) => toast(message, { kind: "success" })} />
  </div>
  <div class="flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-line px-5 py-2 text-body-2 md:px-6">
    <span class="text-muted">Or add by hand:</span>
    {#each SECTIONS as section (section.id)}
      <button
        type="button"
        onclick={() => jump(section.id)}
        class="-mx-1 min-h-10 rounded-tag px-1 font-medium text-accent-ink underline-offset-2 hover:underline"
      >
        {section.label}
      </button>
    {/each}
  </div>
</section>
