<script lang="ts">
  import { ChevronDown, ChevronsUpDown } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { HTMLSelectAttributes } from "svelte/elements";
  import type { Option } from "./types";

  type Props = Omit<HTMLSelectAttributes, "class" | "value" | "size"> & {
    value?: string;
    options?: readonly Option[];
    /** Shown while `value` is "" (rendered as a hidden empty option). */
    placeholder?: string;
    label: string;
    hideLabel?: boolean;
    /** field: the 40px rail field; compact: the 36px toolbar dropdown (Figma "Show", "Department"). */
    variant?: "field" | "compact";
    /** Custom <option>/<optgroup> content instead of `options`. */
    children?: Snippet;
    class?: string;
  };

  let {
    value = $bindable(""),
    options = [],
    placeholder,
    label,
    hideLabel = true,
    variant = "field",
    disabled = false,
    id: providedId,
    children,
    class: className = "",
    ...rest
  }: Props = $props();

  const generated = $props.id();
  const id = $derived(providedId ?? generated);
  const empty = $derived(value === "");
</script>

<div class={className}>
  <label for={id} class={hideLabel ? "sr-only" : "mb-1.5 block text-label-lg text-ink"}>{label}</label>
  <div class="relative">
    <select
      {id}
      bind:value
      {disabled}
      class="focus-field w-full appearance-none bg-surface hover:border-muted disabled:border-line disabled:text-faint {variant ===
      'field'
        ? 'h-10 rounded-field border border-line-strong pr-11 pl-4 text-body-2'
        : 'h-9 rounded-tag border-[0.5px] border-muted pr-10 pl-3.5 text-body-2'} {empty
        ? 'text-muted'
        : 'text-ink'}"
      {...rest}
    >
      {#if placeholder !== undefined}
        <option value="" disabled hidden>{placeholder}</option>
      {/if}
      {#if children}
        {@render children()}
      {:else}
        {#each options as option (option.value)}
          <option value={option.value} disabled={option.disabled}>{option.label}</option>
        {/each}
      {/if}
    </select>
    {#if variant === "field"}
      <ChevronsUpDown
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 right-4 size-5 -translate-y-1/2 {disabled
          ? 'text-faint'
          : 'text-ink'}"
      />
    {:else}
      <ChevronDown
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 right-3 size-5 -translate-y-1/2 text-ink"
      />
    {/if}
  </div>
</div>
