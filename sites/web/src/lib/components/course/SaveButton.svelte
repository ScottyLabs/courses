<script lang="ts">
  import { Star } from "@lucide/svelte";
  import { codeOf } from "$lib/core/requisite-format";
  import { saved } from "$lib/state";

  let {
    courseId,
    size = "md",
    class: className = "",
  }: {
    /** Five-digit course id, such as "15122". */
    courseId: string;
    /** sm 32px (rows); md 40px with a 28px star (cards, as in the Figma). */
    size?: "sm" | "md";
    class?: string;
  } = $props();

  const on = $derived(saved.has(courseId));
  const code = $derived(codeOf(courseId));
</script>

<button
  type="button"
  aria-pressed={on}
  aria-label={on ? `Saved ${code}` : `Save ${code}`}
  title={on ? "Saved. Select to remove it from your saved courses." : "Save this course"}
  onclick={(event) => {
    event.stopPropagation();
    saved.toggle(courseId);
  }}
  class="inline-grid shrink-0 place-items-center rounded-full text-ink transition-colors hover:bg-hover active:bg-active {size ===
  'md'
    ? 'size-10'
    : 'size-8'} {className}"
>
  <Star
    aria-hidden="true"
    strokeWidth={1.75}
    class="{size === 'md' ? 'size-7' : 'size-[18px]'} transition-colors {on
      ? 'fill-accent text-accent'
      : 'fill-transparent'}"
  />
</button>
