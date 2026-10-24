<script lang="ts">
  import { page } from "$app/state";
  import { ChevronLeft, BookOpen } from "@lucide/svelte";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";

  const code = $derived(page.params.code ?? "");
</script>

<Title value={code} />

<PageContainer>
  <a
    href="/"
    class="mb-4 inline-flex items-center gap-1 rounded-full py-1 pr-2 text-label-lg text-muted hover:text-ink"
  >
    <ChevronLeft aria-hidden="true" class="size-5" />All courses
  </a>
  <h1 class="tabular text-heading-3 text-strong">{code}</h1>
  <Card padding="lg" class="mt-6">
    <EmptyState
      size="sm"
      icon={BookOpen}
      title="Course details"
      description="The description, requisites, sections and meetings, offering history, FCEs and syllabi for {code} appear here."
    />
  </Card>
</PageContainer>
