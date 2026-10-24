<script lang="ts">
  import { CalendarPlus } from "@lucide/svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let { title }: { title: string } = $props();
</script>

<EmptyState icon={CalendarPlus} {title} description="Your schedules for every semester appear here." />
