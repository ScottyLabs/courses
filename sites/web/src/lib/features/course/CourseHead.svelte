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

  // The page's one description tag (app.html's, or the server's for a course page) says what this course is, and the
  // site's description again on leaving. Adding a second tag would leave crawlers two to choose from.
  const SITE_DESCRIPTION =
    "Search CMU courses, read FCEs, plan schedules and degrees, and see what your friends are taking.";
  $effect(() => {
    const text = summary;
    if (text === "") return;
    let meta = document.head.querySelector<HTMLMetaElement>('meta[name="description"]');
    if (meta === null) {
      meta = document.createElement("meta");
      meta.name = "description";
      document.head.append(meta);
    }
    meta.content = text;
    const tag = meta;
    return () => {
      tag.content = SITE_DESCRIPTION;
    };
  });
</script>

<Title value={title ? `${code} ${title}` : code} />
<svelte:head>
  <link rel="canonical" href={canonical} />
</svelte:head>
