<script lang="ts">
  import { X } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import { focusableWithin, restoreFocus, trapTab } from "./focus";
  import IconButton from "./IconButton.svelte";

  let {
    open = $bindable(false),
    title,
    description,
    size = "md",
    dismissible = true,
    children,
    footer,
    onclose,
    class: className = "",
  }: {
    open?: boolean;
    title: string;
    description?: string;
    /** sm 400px, md 560px, lg 760px wide (always within the viewport). */
    size?: "sm" | "md" | "lg";
    /** Escape, the close button and backdrop clicks close the dialog. Set false for required choices. */
    dismissible?: boolean;
    children: Snippet;
    /** Action buttons, right-aligned. */
    footer?: Snippet;
    onclose?: () => void;
    class?: string;
  } = $props();

  const id = $props.id();
  let dialog = $state<HTMLDialogElement>();
  let previous: Element | null = null;
  const widths = { sm: "max-w-[400px]", md: "max-w-[560px]", lg: "max-w-[760px]" };

  $effect(() => {
    const element = dialog;
    if (element === undefined) return;
    if (open && !element.open) {
      previous = document.activeElement;
      element.showModal();
      const body = element.querySelector<HTMLElement>("[data-dialog-body]");
      (focusableWithin(body ?? element).at(0) ?? body ?? element).focus();
    } else if (!open && element.open) {
      element.close();
    }
  });

  function onclosed(): void {
    open = false;
    restoreFocus(previous);
    previous = null;
    onclose?.();
  }
</script>

<dialog
  bind:this={dialog}
  data-scroll-lock
  aria-labelledby="{id}-title"
  aria-describedby={description ? `${id}-description` : undefined}
  oncancel={(event) => {
    event.preventDefault();
    if (dismissible) open = false;
  }}
  onclose={onclosed}
  onclick={(event) => {
    if (dismissible && event.target === dialog) open = false;
  }}
  onkeydown={(event) => {
    if (dialog) trapTab(event, dialog);
  }}
  class="modal m-auto max-h-[min(88dvh,900px)] w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-card border border-line bg-surface p-0 text-ink shadow-dialog open:flex {widths[
    size
  ]} {className}"
>
  {#if open}
    <header class="flex items-start justify-between gap-4 border-b border-line px-5 py-4 sm:px-6">
      <div class="min-w-0">
        <h2 id="{id}-title" class="text-heading-5 text-strong">{title}</h2>
        {#if description}
          <p id="{id}-description" class="mt-0.5 text-body-2 text-muted">{description}</p>
        {/if}
      </div>
      {#if dismissible}
        <IconButton label="Close" icon={X} size="sm" class="-mr-2" onclick={() => (open = false)} />
      {/if}
    </header>
    <div data-dialog-body tabindex="-1" class="min-h-0 flex-1 overflow-y-auto px-5 py-5 outline-none sm:px-6">
      {@render children()}
    </div>
    {#if footer}
      <footer class="flex flex-wrap justify-end gap-2 border-t border-line px-5 py-4 sm:px-6">
        {@render footer()}
      </footer>
    {/if}
  {/if}
</dialog>

<style>
  .modal {
    opacity: 0;
    translate: 0 8px;
    transition:
      opacity 0.15s ease,
      translate 0.15s ease,
      overlay 0.15s allow-discrete,
      display 0.15s allow-discrete;
  }

  .modal[open] {
    opacity: 1;
    translate: 0 0;
  }

  @starting-style {
    .modal[open] {
      opacity: 0;
      translate: 0 8px;
    }
  }

  .modal::backdrop {
    background: rgb(8 14 20 / 0);
    transition:
      background 0.15s ease,
      overlay 0.15s allow-discrete,
      display 0.15s allow-discrete;
  }

  .modal[open]::backdrop {
    background: rgb(8 14 20 / 0.45);
  }

  @starting-style {
    .modal[open]::backdrop {
      background: rgb(8 14 20 / 0);
    }
  }
</style>
