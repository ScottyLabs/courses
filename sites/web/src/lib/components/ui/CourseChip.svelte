<script lang="ts">
  import { coursePreview } from "$lib/components/course/preview";
  import type { ChipSize, ChipTone } from "./Chip.svelte";
  import Chip from "./Chip.svelte";
  import StatusChip, { type RequisiteStatus } from "./StatusChip.svelte";

  let {
    code,
    title,
    status,
    href,
    link = true,
    tone = "outline",
    size = "md",
    class: className = "",
  }: {
    /** Course code as shown, such as "15-122". */
    code: string;
    /** Optional course title after the bold code (Figma friends card). */
    title?: string;
    /** Requisite state; switches to the status chip look. */
    status?: RequisiteStatus;
    /** Link target; defaults to the course page. */
    href?: string;
    /** Set false for a plain, non-link chip. */
    link?: boolean;
    tone?: ChipTone;
    size?: ChipSize;
    class?: string;
  } = $props();

  const target = $derived(link ? (href ?? `/course/${encodeURIComponent(code)}`) : undefined);
  // Links preview their course by themselves (PreviewLayer); a plain chip is marked for one.
  const attrs = $derived(link ? {} : coursePreview(code));
</script>

{#snippet text()}
  <span class="tabular {title ? 'font-semibold' : ''}">{code}</span>{#if title}&nbsp;<span
      class="font-normal">{title}</span
    >{/if}
{/snippet}

{#if status}
  <StatusChip {status} href={target} {size} {attrs} class={className}>{@render text()}</StatusChip>
{:else}
  <Chip {tone} href={target} {size} {attrs} class={className}>{@render text()}</Chip>
{/if}
