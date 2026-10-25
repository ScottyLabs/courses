<script lang="ts">
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import AvatarGroup from "$lib/components/ui/AvatarGroup.svelte";
  import Popover from "$lib/components/ui/Popover.svelte";
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
  {@const label = people.length === 1 ? "1 friend taking this course" : `${people.length} friends taking this course`}
  <!-- A button, so touch and keyboard users can see who as well as hover can. -->
  <div class={className}>
    <Popover label="Friends taking this course" placement="bottom-end" class="w-64 p-3">
      {#snippet trigger(props)}
        <button
          type="button"
          {...props}
          aria-label="{label}: {names.join(', ')}"
          title={names.join(", ")}
          class="inline-flex min-h-10 items-center rounded-full px-1 transition-colors hover:bg-hover"
        >
          <AvatarGroup {people} {max} {size} decorative />
        </button>
      {/snippet}
      <p class="text-label-lg text-ink">{label}</p>
      <ul class="mt-2 flex flex-col gap-2">
        {#each names as name (name)}
          <li class="flex min-w-0 items-center gap-2 text-body-2 text-ink">
            <Avatar {name} size="xs" tone="soft" letters={1} decorative />
            <span class="truncate">{name}</span>
          </li>
        {/each}
      </ul>
    </Popover>
  </div>
{/if}
