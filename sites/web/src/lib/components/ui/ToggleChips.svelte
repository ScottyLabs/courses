<script lang="ts" module>
  import type { Option } from "./types";

  export interface ToggleChip extends Option {
    /** The full name, for a label shortened to fit ("Full" for "Full semester"): the tooltip and accessible name. */
    fullLabel?: string;
  }
</script>

<script lang="ts">
  import { togglePill } from "./styles";

  let {
    options,
    value = $bindable([]),
    multiple = true,
    label,
    stretch = false,
    class: className = "",
  }: {
    options: readonly ToggleChip[];
    /** Selected option values. With `multiple` false it holds at most one. */
    value?: string[];
    multiple?: boolean;
    /** Accessible name for the group. */
    label: string;
    /** Make every pill the same width, filling the row. */
    stretch?: boolean;
    class?: string;
  } = $props();

  function toggle(optionValue: string): void {
    if (value.includes(optionValue)) value = value.filter((item) => item !== optionValue);
    else value = multiple ? [...value, optionValue] : [optionValue];
  }
</script>

<div role="group" aria-label={label} class="flex flex-wrap gap-2 {className}">
  {#each options as option (option.value)}
    <button
      type="button"
      class="{togglePill} whitespace-nowrap {stretch ? 'flex-1' : 'px-4'}"
      aria-pressed={value.includes(option.value)}
      aria-label={option.fullLabel}
      title={option.fullLabel}
      disabled={option.disabled}
      onclick={() => toggle(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>
