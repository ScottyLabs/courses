<script lang="ts">
  import { Check, Plus } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { schedulesPath } from "$lib/core/routes";
  import type { CourseSchedule } from "./course-schedule.svelte";

  let { sched, class: className = "" }: { sched: CourseSchedule; class?: string } = $props();

  const entry = $derived(sched.entry);
  const name = $derived(sched.scheduleName ?? "your schedule");
</script>

<div class="flex flex-col gap-1 {className}">
  <div class="flex flex-wrap items-center gap-2" aria-live="polite">
    {#if entry}
      <span class="inline-flex min-h-10 items-center gap-1.5 text-label-lg text-ok">
        <Check aria-hidden="true" class="size-5 shrink-0" />
        In {name}
        {#if entry.entry.hidden}<span class="font-normal text-muted">(hidden)</span>{/if}
        {#if entry.entry.passFail && sched.course?.passFail !== true}<span class="font-normal text-muted">(pass/fail)</span>{/if}
      </span>
      <Button variant="ghost" size="sm" onclick={() => sched.remove()}>Remove</Button>
      <Button variant="ghost" size="sm" href={schedulesPath(sched.termId, sched.scheduleId)}>Open schedule</Button>
    {:else}
      <Button variant="primary" icon={Plus} onclick={() => sched.addDefault()}>
        Add to {sched.scheduleName ?? "schedule"}
      </Button>
    {/if}
  </div>
  {#if entry?.needsChild}
    <p class="text-body-2 text-warn">Pick a recitation in the sections below to finish this course's schedule.</p>
  {/if}
</div>
