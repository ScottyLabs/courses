<script lang="ts">
  import { Check } from "@lucide/svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  let { title }: { title: string } = $props();
</script>

<EmptyState icon={Check} {title} description="Your academic record appears here: courses taken, transfer credit, exams and waivers, imported from SIO or Stellic." />
