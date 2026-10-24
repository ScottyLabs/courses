<script lang="ts">
  import { page } from "$app/state";
  import Title from "$lib/components/shell/Title.svelte";

  let {
    code,
    title,
    description,
  }: {
    code: string;
    title?: string;
    /** The catalog description; trimmed for the meta tag. */
    description?: string;
  } = $props();

  const LIMIT = 160;
  const summary = $derived.by(() => {
    const text = (description ?? "").replaceAll(/\s+/gu, " ").trim();
    if (text.length <= LIMIT) return text;
    const cut = text.slice(0, LIMIT - 1);
    return `${cut.slice(0, Math.max(cut.lastIndexOf(" "), LIMIT - 20))}\u2026`;
  });
  // One address per course across semesters, as the server renders it (decision 25).
  const canonical = $derived(`${page.url.origin}/course/${encodeURIComponent(code)}`);
</script>

<Title value={title ? `${code} ${title}` : code} />
<svelte:head>
  {#if summary}<meta name="description" content={summary} />{/if}
  <link rel="canonical" href={canonical} />
</svelte:head>
