<script lang="ts">
  import { page } from "$app/state";
  import { parseCourseCode } from "$lib/core/data";
  import CoursePage from "$lib/features/course/CoursePage.svelte";

  // "15-122", "15122" or "15 122"; anything else isn't a course number.
  const courseId = $derived.by(() => {
    const digits = parseCourseCode(page.params.code ?? "");
    return /^\d{5}$/u.test(digits) ? digits : "";
  });
  const requestedTerm = $derived(page.url.searchParams.get("term"));
</script>

{#key courseId}
  <CoursePage {courseId} {requestedTerm} />
{/key}
