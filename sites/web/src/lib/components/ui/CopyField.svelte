<!--
  Read-only text with a Copy button: share links, invite links, scripts. When the browser refuses the clipboard, the
  text is selected so it can be copied by hand.
-->
<script lang="ts">
  import { Check, Copy } from "@lucide/svelte";
  import Button, { type ButtonVariant } from "./Button.svelte";

  let {
    value,
    label,
    rows = 1,
    copyLabel = "Copy",
    variant = "outline",
    hint,
    oncopied,
  }: {
    value: string;
    label: string;
    /** More than one shows a text area. */
    rows?: number;
    copyLabel?: string;
    /** The Copy button's look. */
    variant?: ButtonVariant;
    /** A note under the field, such as when a link expires. */
    hint?: string;
    oncopied?: () => void;
  } = $props();

  const id = $props.id();
  let field = $state<HTMLInputElement | HTMLTextAreaElement>();
  let status = $state<"idle" | "copied" | "blocked">("idle");

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(value);
      status = "copied";
      oncopied?.();
      setTimeout(() => {
        if (status === "copied") status = "idle";
      }, 2000);
    } catch {
      status = "blocked";
      field?.focus();
      field?.select();
    }
  }
</script>

<div class="flex flex-col gap-1.5">
  <label for={id} class="text-label-lg text-ink">{label}</label>
  <div class="flex flex-col gap-2 sm:flex-row sm:items-start">
    {#if rows > 1}
      <textarea
        bind:this={field}
        {id}
        readonly
        {rows}
        {value}
        aria-describedby={hint === undefined ? undefined : `${id}-hint`}
        onfocus={(event) => event.currentTarget.select()}
        class="focus-field min-w-0 flex-1 resize-y rounded-field border border-line-strong bg-surface-2 p-2 font-mono text-label-md font-normal text-ink"
      ></textarea>
    {:else}
      <input
        bind:this={field}
        {id}
        readonly
        {value}
        aria-describedby={hint === undefined ? undefined : `${id}-hint`}
        onfocus={(event) => event.currentTarget.select()}
        class="focus-field h-10 min-w-0 flex-1 rounded-field border border-line-strong bg-surface-2 px-3 text-body-2 text-ink"
      />
    {/if}
    <Button {variant} icon={status === "copied" ? Check : Copy} onclick={copy} class="self-start">
      {status === "copied" ? "Copied" : copyLabel}
    </Button>
  </div>
  {#if hint !== undefined}
    <p id="{id}-hint" class="text-label-md font-normal text-muted">{hint}</p>
  {/if}
  <p class="text-label-md font-normal text-muted" aria-live="polite">
    {#if status === "blocked"}
      Your browser blocked copying. The text is selected: copy it with your keyboard.
    {:else if status === "copied"}
      <span class="sr-only">Copied</span>
    {/if}
  </p>
</div>
