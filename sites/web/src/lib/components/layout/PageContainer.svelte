<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    width = "default",
    children,
    class: className = "",
  }: {
    /** narrow 720px for forms and invites; default 1100px; wide 1400px. */
    width?: "narrow" | "default" | "wide";
    children: Snippet;
    class?: string;
  } = $props();

  const widths = { narrow: "max-w-[720px]", default: "max-w-[1100px]", wide: "max-w-[1400px]" };
</script>

<div class="mx-auto w-full flex-1 px-4 py-6 sm:px-6 lg:px-8 lg:py-10 {widths[width]} {className}">
  {@render children()}
</div>
