<script lang="ts">
  // A titled block of the record (Courseweave's Section in features/record/shared.tsx).
  import type { Snippet } from "svelte";

  let {
    id,
    title,
    description,
    aside,
    children,
  }: {
    /** The heading's id; also the target of "add by hand" jump links. */
    id: string;
    title: string;
    description?: string;
    /** Controls right of the heading. */
    aside?: Snippet;
    children: Snippet;
  } = $props();
</script>

<section aria-labelledby={id} class="flex scroll-mt-(--header-h) flex-col gap-4">
  <div class="flex flex-wrap items-end justify-between gap-x-4 gap-y-2">
    <div class="min-w-0">
      <h2 {id} class="scroll-mt-[calc(var(--header-h)+1rem)] text-heading-5 text-strong">{title}</h2>
      {#if description}<p class="mt-1 max-w-[68ch] text-body-2 text-ink">{description}</p>{/if}
    </div>
    {@render aside?.()}
  </div>
  {@render children()}
</section>
