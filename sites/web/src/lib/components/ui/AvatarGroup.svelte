<script lang="ts">
  import Avatar from "./Avatar.svelte";

  let {
    people,
    max = 2,
    size = "sm",
    label = "Friends",
    decorative = false,
    class: className = "",
  }: {
    people: readonly { name: string; src?: string }[];
    /** Avatars shown before the "+N" count. */
    max?: number;
    size?: "xs" | "sm";
    /** Describes who these people are, such as "Friends taking this course". */
    label?: string;
    /** Hide from assistive technology, when a control around it names the people. */
    decorative?: boolean;
    class?: string;
  } = $props();

  const shown = $derived(people.slice(0, max));
  const extra = $derived(people.length - shown.length);
  const description = $derived.by(() => {
    const names = shown.map((person) => person.name).join(", ");
    return extra > 0 ? `${label}: ${names} and ${extra} more` : `${label}: ${names}`;
  });
</script>

{#if people.length > 0}
  <div
    role={decorative ? undefined : "img"}
    aria-label={decorative ? undefined : description}
    aria-hidden={decorative ? "true" : undefined}
    title={decorative ? undefined : people.map((person) => person.name).join(", ")}
    class="inline-flex items-center {size === 'sm' ? 'gap-1.5' : 'gap-1'} {className}"
  >
    {#each shown as person, index (index)}
      <Avatar name={person.name} src={person.src} {size} tone="soft" letters={1} decorative />
    {/each}
    {#if extra > 0}
      <span
        aria-hidden="true"
        class="tabular text-label-md text-ink {size === 'sm'
          ? 'inline-flex h-5 items-center rounded-full bg-paper px-1.5'
          : ''}">+{extra}</span
      >
    {/if}
  </div>
{/if}
