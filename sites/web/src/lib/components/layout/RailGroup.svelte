<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    children,
    class: className = "",
  }: {
    /** Group heading (Figma "Aggregate FCEs"). Groups after the first get a divider above. */
    title?: string;
    children: Snippet;
    class?: string;
  } = $props();
</script>

<section
  class="border-line px-5 pb-8 not-first:border-t not-first:pt-8 sm:px-6 min-[90rem]:px-[31px] {className}"
>
  {#if title}
    <h2 class="mb-3 text-heading-5 text-ink">{title}</h2>
  {/if}
  <div class="flex flex-col gap-6">
    {@render children()}
  </div>
</section>
