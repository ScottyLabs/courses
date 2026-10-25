<script lang="ts">
  // A planned course's requisite state as an icon; hovering, focusing or tapping it shows the details in a tooltip.
  // Waiving requisites is in the course's options menu.
  import RequisiteLevelIcon from "$lib/components/course/RequisiteLevelIcon.svelte";
  import Tooltip from "$lib/components/ui/Tooltip.svelte";
  import type { CheckLevel } from "$lib/core/requisites";

  let { level, text, code }: { level: CheckLevel; text: string; code: string } = $props();
</script>

<Tooltip {text} delay={150}>
  {#snippet trigger(props)}
    <button
      type="button"
      {...props}
      aria-label="{code} requisites"
      class="grid size-10 shrink-0 cursor-default place-items-center rounded-full hover:bg-hover md:size-7"
    >
      <RequisiteLevelIcon {level} class="size-4" />
    </button>
  {/snippet}
</Tooltip>
