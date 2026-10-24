<script lang="ts">
  let {
    size = "md",
    label = "Loading",
    decorative = false,
    class: className = "",
  }: {
    size?: "sm" | "md" | "lg";
    /** Announced to screen readers unless `decorative`. */
    label?: string;
    /** Hide from assistive technology, for spinners inside a busy button. */
    decorative?: boolean;
    class?: string;
  } = $props();

  const sizes = { sm: "size-4 border-2", md: "size-5 border-2", lg: "size-8 border-[3px]" };
</script>

<span
  role={decorative ? undefined : "status"}
  aria-label={decorative ? undefined : label}
  aria-hidden={decorative ? "true" : undefined}
  class="inline-block shrink-0 animate-spin rounded-full border-line-strong border-t-accent {sizes[
    size
  ]} {className}"
></span>
