<script lang="ts">
  import { Inbox } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { IconComponent } from "./types";

  let {
    title,
    description,
    icon: Icon = Inbox,
    size = "md",
    headingLevel = 2,
    class: className = "",
    children,
  }: {
    title: string;
    description?: string;
    icon?: IconComponent;
    /** sm for inside cards and panels; md for a page's main area. */
    size?: "sm" | "md";
    headingLevel?: 1 | 2 | 3 | 4;
    class?: string;
    /** Actions, such as a sign-in or reset button. */
    children?: Snippet;
  } = $props();
</script>

<div
  class="flex flex-col items-center text-center {size === 'md' ? 'gap-3 px-4 py-16' : 'gap-2 px-4 py-8'} {className}"
>
  <span
    class="grid place-items-center rounded-full bg-surface-2 text-muted {size === 'md'
      ? 'size-14 [&_svg]:size-6'
      : 'size-10 [&_svg]:size-5'}"
  >
    <Icon aria-hidden="true" />
  </span>
  <svelte:element
    this={`h${headingLevel}`}
    class="text-strong {size === 'md' ? 'text-heading-5' : 'text-heading-6'}"
  >
    {title}
  </svelte:element>
  {#if description}
    <p class="max-w-md text-body-2 text-muted">{description}</p>
  {/if}
  {#if children}
    <div class="mt-2 flex flex-wrap items-center justify-center gap-2">{@render children()}</div>
  {/if}
</div>
