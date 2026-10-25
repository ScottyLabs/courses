<script lang="ts">
  import { Search, X } from "@lucide/svelte";
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = Omit<HTMLInputAttributes, "class" | "value" | "type" | "size"> & {
    value?: string;
    /** Accessible name for the field (the placeholder is not a label). */
    label?: string;
    /** underline: the Figma page search (no box, rule underneath); field: a rounded box for panels. */
    variant?: "underline" | "field";
    /** Called with the query when Enter is pressed. */
    onsubmit?: (value: string) => void;
    ref?: HTMLInputElement;
    class?: string;
  };

  let {
    value = $bindable(""),
    label = "Search",
    variant = "underline",
    placeholder = "Search",
    onsubmit,
    ref = $bindable(),
    class: className = "",
    ...rest
  }: Props = $props();

  function clear(): void {
    value = "";
    ref?.focus();
  }

  function onkeydown(event: KeyboardEvent): void {
    if (event.key === "Enter") onsubmit?.(value);
    else if (event.key === "Escape" && value !== "") {
      event.preventDefault();
      value = "";
    }
  }
</script>

<!-- Focus shows on the field itself: the underline variant grows a 2px accent rule from the center over its line,
     the boxed one takes the field style (focus-field). -->
<div
  role="search"
  class="group relative flex items-center {variant === 'underline'
    ? 'h-12 border-b border-line-strong px-3 after:pointer-events-none after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:scale-x-0 after:bg-accent after:transition-transform after:duration-200 after:ease-out focus-within:after:scale-x-100 lg:h-[50px] lg:px-4'
    : 'focus-field h-10 rounded-full border border-line-strong bg-surface px-3'} {className}"
>
  <Search
    aria-hidden="true"
    class="shrink-0 text-muted {variant === 'underline' ? 'size-6' : 'size-5'}"
  />
  <input
    bind:this={ref}
    bind:value
    type="search"
    aria-label={label}
    {placeholder}
    {onkeydown}
    autocomplete="off"
    spellcheck="false"
    class="h-full min-w-0 flex-1 bg-transparent px-2.5 text-ellipsis text-ink outline-none placeholder:text-muted [&::-webkit-search-cancel-button]:appearance-none {variant ===
    'underline'
      ? 'text-body-1 lg:text-heading-4'
      : 'text-body-2'}"
    {...rest}
  />
  {#if value !== ""}
    <button
      type="button"
      aria-label="Clear search"
      title="Clear search"
      onclick={clear}
      class="grid size-8 shrink-0 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink"
    >
      <X aria-hidden="true" class="size-4" />
    </button>
  {/if}
</div>
