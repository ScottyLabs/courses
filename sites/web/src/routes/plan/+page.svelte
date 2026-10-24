<script lang="ts">
  import { page } from "$app/state";
  import Title from "$lib/components/shell/Title.svelte";
  import PlanPage from "$lib/features/plan/PlanPage.svelte";
</script>

<Title value="Degree plan" />

<PlanPage planId={page.url.searchParams.get("plan")} termParam={page.url.searchParams.get("term")} />
