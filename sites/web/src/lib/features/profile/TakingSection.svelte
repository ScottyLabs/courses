<script lang="ts">
  import { Calendar1 } from "@lucide/svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let { title }: { title: string } = $props();
</script>

<EmptyState icon={Calendar1} {title} description="This semester's courses from your record appear here, with their sections and times." />
