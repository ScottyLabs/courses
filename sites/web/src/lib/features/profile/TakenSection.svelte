<script lang="ts">
  // Profile "Courses Taken": the academic record. Courses taken, exam and transfer credit, placements and overrides,
  // imported from SIO or Stellic or added by hand; prerequisite checks and the degree plan read it. Courseweave's
  // record page (features/record/RecordPage.tsx), with courses shown as the Figma's Courses Taken page shows them.
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { personal, record } from "$lib/state";
  import ExamCredit from "$lib/features/record/ExamCredit.svelte";
  import ProfileTotals from "$lib/features/record/ProfileTotals.svelte";
  import RecordActions from "$lib/features/record/RecordActions.svelte";
  import { recordData } from "$lib/features/record/record-data.svelte";
  import RecordEmptyStart from "$lib/features/record/RecordEmptyStart.svelte";
  import TakenCourses from "$lib/features/record/TakenCourses.svelte";
  import TransferCredit from "$lib/features/record/TransferCredit.svelte";
  import Waivers from "$lib/features/record/Waivers.svelte";

  // The profile page passes the section's title; its band shows it.
  let { title }: { title: string } = $props();

  const data = recordData();
  const semester = $derived(data.term?.meta.name);
</script>

{#if personal.mode === "loading"}
  <div class="flex justify-center py-16"><Spinner size="lg" label="Loading your record" /></div>
{:else}
<div class="flex flex-col gap-10" aria-label={title} role="region">
  <div class="flex flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <p class="max-w-[62ch] text-body-2 text-ink">
        What you've already earned: CMU courses, exam and transfer credit, and placements. Prerequisite checks in your
        schedules and your degree plan use it.
      </p>
      <RecordActions />
    </div>
    {#if record.isEmpty}
      <RecordEmptyStart />
    {/if}
    <ProfileTotals {data} />
  </div>
  <TakenCourses {data} />
  <ExamCredit {data} />
  <TransferCredit {data} />
  <Waivers {data} />
  <p class="border-t border-line pt-4 text-label-md font-normal text-muted">
    Your record is saved to your account; export a backup to keep a copy of your own.
    {#if semester}Course titles and units come from the {semester} catalog.{/if}
    Exam credit charts are from the University Registrar.
  </p>
</div>
{/if}
