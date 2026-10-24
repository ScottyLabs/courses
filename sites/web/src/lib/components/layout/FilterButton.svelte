<script lang="ts">
  import { ListFilter } from "@lucide/svelte";
  import { getRail } from "./rail-context";

  let { class: className = "" }: { class?: string } = $props();
  const rail = getRail();
</script>

{#if rail?.asSheet}
  <button
    type="button"
    aria-haspopup="dialog"
    onclick={() => rail.open()}
    class="inline-flex h-8 items-center gap-1.5 rounded-full text-label-md transition-colors {rail.count > 0
      ? 'bg-strong px-3 text-surface'
      : '-ml-1 px-1 text-strong hover:bg-hover'} {className}"
  >
    <ListFilter aria-hidden="true" class="size-[18px]" />
    {rail.label}
    {#if rail.count > 0}
      <span class="tabular"><span class="sr-only">(</span>{rail.count}<span class="sr-only">{" active)"}</span></span>
    {/if}
  </button>
{/if}
