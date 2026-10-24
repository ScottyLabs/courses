<script lang="ts">
  import AvatarGroup from "$lib/components/ui/AvatarGroup.svelte";
  import { friendsTaking } from "./data.svelte";

  let {
    courseId,
    termId,
    size = "sm",
    max = 2,
    class: className = "",
  }: {
    courseId: string;
    /** The semester they take it in. */
    termId: string;
    size?: "xs" | "sm";
    max?: number;
    class?: string;
  } = $props();

  // Signed out, or nobody shares this course: nothing to show.
  const names = $derived(friendsTaking(termId).value?.[courseId] ?? []);
  const people = $derived(names.map((name) => ({ name })));
</script>

{#if people.length > 0}
  <AvatarGroup
    {people}
    {max}
    {size}
    label={people.length === 1 ? "1 friend taking this course" : `${people.length} friends taking this course`}
    class={className}
  />
{/if}
