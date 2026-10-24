<script lang="ts" module>
  export type RequisiteStatus = "satisfied" | "planned" | "in-progress" | "unsatisfied";
</script>

<script lang="ts">
  import { ArrowRight, Check, X } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import Chip, { type ChipSize } from "./Chip.svelte";

  let {
    status,
    label,
    href,
    size = "md",
    class: className = "",
    children,
  }: {
    /** satisfied: check on green; planned / in-progress: arrow on yellow; unsatisfied: x on gray. */
    status: RequisiteStatus;
    label?: string;
    href?: string;
    size?: ChipSize;
    class?: string;
    children?: Snippet;
  } = $props();

  const looks = {
    satisfied: { tone: "ok", icon: Check, text: "satisfied" },
    planned: { tone: "warn", icon: ArrowRight, text: "planned" },
    "in-progress": { tone: "warn", icon: ArrowRight, text: "in progress" },
    unsatisfied: { tone: "soft", icon: X, text: "not satisfied" },
  } as const;
  const look = $derived(looks[status]);
</script>

<Chip tone={look.tone} icon={look.icon} {href} {size} class={className}>
  {#if children}{@render children()}{:else}{label}{/if}<span class="sr-only">, {look.text}</span>
</Chip>
