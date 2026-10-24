<script lang="ts">
  // "Import from SIO or Stellic": reads an SIO academic record PDF or a Stellic plan PDF in the browser and lets the
  // student choose what to add to the record; a Stellic plan also becomes a degree plan. Courseweave's
  // features/record/SioImport.tsx. (Placeholder until the record feature lands.)
  import { FileText } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import Button, { type ButtonSize, type ButtonVariant } from "$lib/components/ui/Button.svelte";

  let {
    onimported,
    onplancreated,
    variant = "outline",
    size = "md",
    label = "Import from SIO or Stellic",
    hidden = false,
    trigger,
  }: {
    /** After items were added to the record or a plan was created, with a sentence saying what happened. */
    onimported?: (message: string) => void;
    /** After a Stellic plan became a new degree plan. */
    onplancreated?: (planId: string) => void;
    variant?: ButtonVariant;
    size?: ButtonSize;
    label?: string;
    /** Render no button; open the file chooser with `choose()` (from a menu item). */
    hidden?: boolean;
    /** A custom trigger in place of the default button. */
    trigger?: Snippet<[{ choose: () => void; reading: boolean }]>;
  } = $props();

  let input = $state<HTMLInputElement>();
  const reading = false;

  /** Opens the file chooser. */
  export function choose(): void {
    input?.click();
  }

  function onchange(): void {
    if (input) input.value = "";
    onimported?.("");
    onplancreated?.("");
  }
</script>

{#if trigger}
  {@render trigger({ choose, reading })}
{:else if !hidden}
  <Button {variant} {size} icon={FileText} onclick={choose}>{label}</Button>
{/if}
<input bind:this={input} type="file" accept="application/pdf,.pdf" class="hidden" {onchange} />
