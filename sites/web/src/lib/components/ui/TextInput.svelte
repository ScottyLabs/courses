<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import type { IconComponent } from "./types";

  type Props = Omit<HTMLInputAttributes, "class" | "value" | "size"> & {
    value?: string | number | null;
    /** Visible label above the field; use `hideLabel` to keep it for screen readers only. */
    label: string;
    hideLabel?: boolean;
    /** Help text under the field. */
    hint?: string;
    /** Error text under the field; also marks the field invalid. */
    error?: string;
    icon?: IconComponent;
    /** md: 40px rail field; sm: 28px compact field (Figma "Semesters to show"). */
    size?: "sm" | "md";
    ref?: HTMLInputElement;
    class?: string;
  };

  let {
    value = $bindable(""),
    label,
    hideLabel = false,
    hint,
    error,
    icon: Icon,
    size = "md",
    type = "text",
    id: providedId,
    ref = $bindable(),
    class: className = "",
    ...rest
  }: Props = $props();

  const generated = $props.id();
  const id = $derived(providedId ?? generated);
  const messageId = $derived(`${id}-message`);
  const sizes = {
    md: "h-10 rounded-field px-4 text-body-2",
    sm: "h-7 rounded-[8px] px-4 text-label-md font-normal",
  };
</script>

<div class={className}>
  <label for={id} class={hideLabel ? "sr-only" : "mb-1.5 block text-label-lg text-ink"}>{label}</label>
  <div class="relative">
    {#if Icon}
      <Icon
        aria-hidden="true"
        class="pointer-events-none absolute top-1/2 left-3.5 size-5 -translate-y-1/2 text-muted"
      />
    {/if}
    <input
      bind:this={ref}
      bind:value
      {id}
      {type}
      aria-invalid={error ? "true" : undefined}
      aria-describedby={error || hint ? messageId : undefined}
      class="focus-field w-full border bg-surface text-ink placeholder:text-muted hover:border-muted disabled:border-line disabled:text-faint {sizes[
        size
      ]} {error ? 'border-danger' : 'border-line-strong'} {Icon ? 'pl-11' : ''}"
      {...rest}
    />
  </div>
  {#if error}
    <p id={messageId} class="mt-1 text-label-md text-danger">{error}</p>
  {:else if hint}
    <p id={messageId} class="mt-1 text-label-md font-normal text-muted">{hint}</p>
  {/if}
</div>
