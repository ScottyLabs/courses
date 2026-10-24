<script lang="ts">
  import { CalendarDays } from "@lucide/svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
</script>

<Title value="Shared schedule" />

<PageContainer>
  <h1 class="text-heading-3 text-strong">Shared schedule</h1>
  <p class="mt-1 text-body-2 text-muted">A read-only schedule someone shared with you.</p>
  <Card padding="lg" class="mt-6">
    <EmptyState
      size="sm"
      icon={CalendarDays}
      title="Schedule"
      description="The shared schedule's week calendar and course list appear here, with an option to copy it into your schedules."
    />
  </Card>
</PageContainer>
