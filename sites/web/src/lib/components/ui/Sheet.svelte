<script lang="ts">
  import { ChevronLeft, X } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import { focusableWithin, restoreFocus, trapTab } from "./focus";
  import IconButton from "./IconButton.svelte";

  let {
    open = $bindable(false),
    title,
    side = "right",
    backLabel,
    children,
    footer,
    onclose,
    class: className = "",
  }: {
    open?: boolean;
    title: string;
    /** Edge the sheet slides from. Left and right sheets fill the screen on phones. */
    side?: "left" | "right" | "bottom";
    /**
     * Phone-style header: a back button with this text on the left and the title centered
     * (Figma mobile filter, "< Search  Filter"). Without it the header has a close button.
     */
    backLabel?: string;
    /** Content stays mounted while closed, so its state survives reopening. */
    children: Snippet;
    footer?: Snippet;
    onclose?: () => void;
    class?: string;
  } = $props();

  const id = $props.id();
  let dialog = $state<HTMLDialogElement>();
  let previous: Element | null = null;

  const sides = {
    left: "sheet-left mr-auto h-dvh max-h-dvh w-full sm:w-[min(26rem,calc(100vw-3rem))]",
    right: "sheet-right ml-auto h-dvh max-h-dvh w-full sm:w-[min(26rem,calc(100vw-3rem))]",
    bottom: "sheet-bottom mt-auto max-h-[90dvh] w-full max-w-none rounded-t-card",
  };

  $effect(() => {
    const element = dialog;
    if (element === undefined) return;
    if (open && !element.open) {
      previous = document.activeElement;
      element.showModal();
      const body = element.querySelector<HTMLElement>("[data-sheet-body]");
      (focusableWithin(body ?? element).at(0) ?? element).focus({ preventScroll: true });
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
  oncancel={(event) => {
    event.preventDefault();
    open = false;
  }}
  onclose={onclosed}
  onclick={(event) => {
    if (event.target === dialog) open = false;
  }}
  onkeydown={(event) => {
    if (dialog) trapTab(event, dialog);
  }}
  class="sheet m-0 max-w-none flex-col overflow-hidden border-line bg-surface p-0 text-ink shadow-dialog open:flex {sides[
    side
  ]} {className}"
>
  {#if backLabel !== undefined}
    <header class="grid shrink-0 grid-cols-[1fr_auto_1fr] items-center gap-2 px-4 pt-6 pb-4">
      <button
        type="button"
        onclick={() => (open = false)}
        class="inline-flex items-center gap-1 justify-self-start rounded-full py-1 pr-2 text-body-1 text-strong hover:bg-hover"
      >
        <ChevronLeft aria-hidden="true" class="size-6" />{backLabel}
      </button>
      <h2 id="{id}-title" class="text-heading-3 text-strong">{title}</h2>
      <span></span>
    </header>
  {:else}
    <header class="flex shrink-0 items-center justify-between gap-4 border-b border-line px-5 py-3">
      <h2 id="{id}-title" class="text-heading-5 text-strong">{title}</h2>
      <IconButton label="Close" icon={X} size="sm" class="-mr-2" onclick={() => (open = false)} />
    </header>
  {/if}
  <div data-sheet-body class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
    {@render children()}
  </div>
  {#if footer}
    <footer class="flex shrink-0 flex-wrap justify-end gap-2 border-t border-line px-5 py-3">
      {@render footer()}
    </footer>
  {/if}
</dialog>

<style>
  .sheet {
    transition:
      translate 0.2s ease,
      overlay 0.2s allow-discrete,
      display 0.2s allow-discrete;
  }

  .sheet-left {
    translate: -100% 0;
  }

  .sheet-right {
    translate: 100% 0;
  }

  .sheet-bottom {
    translate: 0 100%;
  }

  .sheet[open] {
    translate: 0 0;
  }

  @starting-style {
    .sheet-left[open] {
      translate: -100% 0;
    }

    .sheet-right[open] {
      translate: 100% 0;
    }

    .sheet-bottom[open] {
      translate: 0 100%;
    }
  }

  .sheet::backdrop {
    background: rgb(8 14 20 / 0);
    transition:
      background 0.2s ease,
      overlay 0.2s allow-discrete,
      display 0.2s allow-discrete;
  }

  .sheet[open]::backdrop {
    background: rgb(8 14 20 / 0.45);
  }

  @starting-style {
    .sheet[open]::backdrop {
      background: rgb(8 14 20 / 0);
    }
  }
</style>
