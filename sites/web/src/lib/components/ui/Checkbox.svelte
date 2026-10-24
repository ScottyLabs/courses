<script lang="ts">
  import { Check, Minus } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { HTMLInputAttributes } from "svelte/elements";

  type Props = Omit<HTMLInputAttributes, "class" | "type" | "checked"> & {
    checked?: boolean;
    /** Mixed state, for "select all" boxes. */
    indeterminate?: boolean;
    label?: string;
    /** Smaller text under the label. */
    description?: string;
    /** Custom label content instead of `label`. */
    children?: Snippet;
    class?: string;
  };

  let {
    checked = $bindable(false),
    indeterminate = $bindable(false),
    label,
    description,
    disabled = false,
    children,
    class: className = "",
    ...rest
  }: Props = $props();
</script>

<label
  class="inline-flex items-center gap-1 text-label-lg text-ink {disabled
    ? 'opacity-50'
    : 'cursor-pointer'} {className}"
>
  <span class="relative grid size-8 shrink-0 place-items-center">
    <input
      type="checkbox"
      bind:checked
      bind:indeterminate
      {disabled}
      class="peer size-4 cursor-[inherit] appearance-none rounded-[4px] border border-ink bg-surface transition-colors checked:border-accent checked:bg-accent indeterminate:border-accent indeterminate:bg-accent"
      {...rest}
    />
    <Check
      aria-hidden="true"
      strokeWidth={3}
      class="pointer-events-none absolute size-3 text-on-accent opacity-0 peer-checked:opacity-100 peer-indeterminate:opacity-0"
    />
    <Minus
      aria-hidden="true"
      strokeWidth={3}
      class="pointer-events-none absolute size-3 text-on-accent opacity-0 peer-indeterminate:opacity-100"
    />
  </span>
  {#if children || label || description}
    <span class="flex flex-col">
      {#if children}{@render children()}{:else}{label}{/if}
      {#if description}<span class="text-label-md font-normal text-muted">{description}</span>{/if}
    </span>
  {/if}
</label>
