<script lang="ts">
  let {
    variant = "block",
    lines = 1,
    class: className = "",
  }: {
    /** `text` draws `lines` bars like a paragraph; `circle` is for avatars. */
    variant?: "block" | "text" | "circle";
    lines?: number;
    class?: string;
  } = $props();
</script>

{#if variant === "text"}
  <div class="flex flex-col gap-2 {className}" aria-hidden="true">
    {#each { length: lines }, index (index)}
      <div
        class="h-3.5 animate-skeleton rounded-full bg-line {index === lines - 1 && lines > 1
          ? 'w-3/5'
          : 'w-full'}"
      ></div>
    {/each}
  </div>
{:else}
  <div
    class="animate-skeleton bg-line {variant === 'circle' ? 'rounded-full' : 'rounded-[8px]'} {className}"
    aria-hidden="true"
  ></div>
{/if}
