<script lang="ts">
  import { ChevronDown } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { IconComponent } from "../ui/types";

  let {
    title,
    icon: Icon,
    open = $bindable(true),
    badge,
    children,
    class: className = "",
  }: {
    /** Section label (Figma "Department", "Units"). */
    title: string;
    icon?: IconComponent;
    /** Expanded; the header toggles it. Collapsed sections show a rule instead of their content. */
    open?: boolean;
    /** Small count after the title, such as how many departments are chosen. */
    badge?: number | string;
    children: Snippet;
    class?: string;
  } = $props();

  const id = $props.id();
</script>

<section class="flex flex-col gap-2 {className}">
  <h3>
    <button
      type="button"
      aria-expanded={open}
      aria-controls={id}
      onclick={() => (open = !open)}
      class="flex w-full items-center justify-between gap-2 rounded-tag text-left text-label-lg text-ink"
    >
      <span class="flex min-w-0 items-center gap-1">
        {#if Icon}<Icon aria-hidden="true" class="size-[18px] shrink-0" />{/if}
        <span class="truncate">{title}</span>
        {#if badge !== undefined && badge !== 0}
          <span class="tabular ml-1 rounded-full bg-line px-1.5 text-label-md">{badge}</span>
        {/if}
      </span>
      <ChevronDown
        aria-hidden="true"
        class="size-5 shrink-0 transition-transform {open ? 'rotate-180' : ''}"
      />
    </button>
  </h3>
  <div {id} hidden={!open} class="flex flex-col gap-2">
    {@render children()}
  </div>
  {#if !open}<div aria-hidden="true" class="h-px bg-line"></div>{/if}
</section>
