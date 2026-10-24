<script lang="ts">
  import { page } from "$app/state";
  import InstructorPage from "$lib/features/instructors/InstructorPage.svelte";
</script>

{#key page.params.id}
  <InstructorPage slug={page.params.id ?? ""} />
{/key}
