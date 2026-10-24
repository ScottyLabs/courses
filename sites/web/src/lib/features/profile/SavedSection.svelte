<script lang="ts">
  import { BookmarkPlus } from "@lucide/svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let { title }: { title: string } = $props();
</script>

<EmptyState icon={BookmarkPlus} {title} description="Courses you save from the course list appear here." />
