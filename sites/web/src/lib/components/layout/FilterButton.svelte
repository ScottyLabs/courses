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
    aria-label={rail.count > 0 ? `${rail.label}, ${rail.count} active` : undefined}
    onclick={() => rail.open()}
    class="inline-flex h-10 items-center gap-1.5 rounded-full text-label-md transition-colors {rail.count > 0
      ? 'bg-strong px-3 text-surface'
      : '-ml-1 px-1 text-strong hover:bg-hover'} {className}"
  >
    <ListFilter aria-hidden="true" class="size-[18px]" />
    {rail.label}
    {#if rail.count > 0}
      <span aria-hidden="true" class="tabular">{rail.count}</span>
    {/if}
  </button>
{/if}
