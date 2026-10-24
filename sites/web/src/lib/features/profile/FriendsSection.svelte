<script lang="ts">
  import { Users } from "@lucide/svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let { title }: { title: string } = $props();
</script>

<EmptyState icon={Users} {title} description="Your friends, friend requests and invite links appear here, with what each friend shares." />
