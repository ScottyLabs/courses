<script lang="ts">
  // A small heading inside an OptionsMenu ("Move to"), with an optional rule above it.
  import type { Snippet } from "svelte";

  let { separated = false, children }: { separated?: boolean; children: Snippet } = $props();
</script>

{#if separated}<div role="separator" class="my-1.5 h-px bg-line"></div>{/if}
<p class="px-3.5 pt-1.5 pb-1 text-label-md text-muted">{@render children()}</p>
