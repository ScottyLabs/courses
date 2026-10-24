<script lang="ts">
  // A planned course's requisite state as an icon button; it shows or hides the issues under the row.
  import RequisiteLevelIcon from "$lib/components/course/RequisiteLevelIcon.svelte";
  import type { CheckLevel } from "$lib/core/requisites";

  let {
    level,
    text,
    code,
    open,
    controls,
    ontoggle,
  }: {
    level: CheckLevel;
    text: string;
    code: string;
    open: boolean;
    controls: string;
    ontoggle: () => void;
  } = $props();
</script>

<button
  type="button"
  onclick={ontoggle}
  aria-expanded={open}
  aria-controls={controls}
  aria-label="{code}: {text}"
  title={text}
  class="grid size-10 shrink-0 place-items-center rounded-full hover:bg-hover md:size-7"
>
  <RequisiteLevelIcon {level} class="size-4" />
</button>
