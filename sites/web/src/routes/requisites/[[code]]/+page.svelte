<script lang="ts">
  import { page } from "$app/state";
  import { parseCourseCode } from "$lib/core/data";
  import RequisitesExplorer from "$lib/features/course/RequisitesExplorer.svelte";

  // `/requisites` starts from the gateway courses; `/requisites/15-122` centers on a course.
  const courseId = $derived.by(() => {
    const code = page.params.code;
    if (code === undefined || code === "") return null;
    const digits = parseCourseCode(code);
    return /^\d{5}$/u.test(digits) ? digits : code;
  });
  const requestedTerm = $derived(page.url.searchParams.get("term"));
</script>

<RequisitesExplorer {courseId} {requestedTerm} />
