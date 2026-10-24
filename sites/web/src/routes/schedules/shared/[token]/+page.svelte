<script lang="ts">
  import { CalendarX2, Copy, LogIn } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { ApiError, api } from "$lib/api";
  import type { SharedSchedule } from "$lib/api/types";
  import PageContainer from "$lib/components/layout/PageContainer.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { catalog, resource, session } from "$lib/data";
  import { schedulesPath } from "$lib/core/routes";
  import { normalizeSchedule } from "$lib/core/sync-merge";
  import { personal, schedules } from "$lib/state";
  import SharedSchedulePreview from "$lib/features/schedules/SharedSchedulePreview.svelte";

  const token = $derived(page.params.token ?? "");

  let shared = $state<SharedSchedule | null>(null);
  let failure = $state<"gone" | "error" | null>(null);
  let message = $state("");

  $effect(() => {
    const current = token;
    shared = null;
    failure = null;
    let live = true;
    api.schedules.shared(current).then(
      (value) => {
        if (live) shared = value;
      },
      (error: unknown) => {
        if (!live) return;
        failure = error instanceof ApiError && error.status === 404 ? "gone" : "error";
        message = error instanceof Error ? error.message : "";
      },
    );
    return () => {
      live = false;
    };
  });

  const term = resource(() => (shared === null ? null : catalog.term(shared.termId)));
  const entries = $derived.by(() => {
    if (shared === null) return [];
    // The same reading as a stored schedule's, so a malformed entry can't break the page.
    return normalizeSchedule("shared", { ...shared, id: "shared", createdAt: 0, updatedAt: 0 })?.entries ?? [];
  });

  function copy(): void {
    if (shared === null || term.value === undefined) return;
    const offered = entries.filter((e) => term.value?.courseById.has(e.courseId) === true);
    const taken = schedules.forTerm(shared.termId).some((s) => s.name === shared?.name);
    const id = schedules.importShared({
      termId: shared.termId,
      name: taken ? `${shared.name} (shared)` : shared.name,
      entries: offered,
    });
    toast(`Saved "${shared.name}" to your schedules.`, { kind: "success" });
    void goto(schedulesPath(shared.termId, id));
  }
</script>

<Title value={shared?.name ?? "Shared schedule"} />

<PageContainer>
  {#if failure === "gone"}
    <EmptyState
      icon={CalendarX2}
      title="This link doesn't work anymore"
      description="Whoever shared it turned the link off or deleted the schedule. Ask them for a new link."
      headingLevel={1}
    >
      <Button href={schedulesPath()}>Go to your schedules</Button>
    </EmptyState>
  {:else if failure === "error"}
    <EmptyState
      icon={CalendarX2}
      title="Couldn't open the shared schedule"
      description="{message} Check your connection and reload the page."
      headingLevel={1}
    />
  {:else if shared === null || (term.value === undefined && term.status !== "error")}
    <div class="flex justify-center py-16"><Spinner size="lg" label="Loading the shared schedule" /></div>
  {:else if term.status === "error" || term.value === undefined}
    <EmptyState
      icon={CalendarX2}
      title="Couldn't load the semester"
      description="This schedule is for a semester this site doesn't have, or the catalog couldn't load. {term.error ??
        ''}"
      headingLevel={1}
    />
  {:else}
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="min-w-0">
        <p class="text-label-md font-normal text-muted">Shared schedule &middot; read only</p>
        <h1 class="text-heading-3 text-strong">{shared.name}</h1>
      </div>
      <Button variant="primary" icon={Copy} onclick={copy} disabled={personal.mode === "loading"}>
        Copy to my schedules
      </Button>
    </div>
    {#if session.status === "signed-out"}
      <div class="mt-4 flex flex-wrap items-center gap-3 rounded-card border border-line bg-surface px-4 py-3">
        <p class="min-w-0 flex-1 text-body-2 text-ink">
          Signed out, a copy is kept in this tab only. Sign in to save it to your account.
        </p>
        <Button size="sm" icon={LogIn} onclick={() => session.signIn()}>Sign in</Button>
      </div>
    {/if}
    <div class="mt-6">
      <SharedSchedulePreview {entries} term={term.value} />
    </div>
  {/if}
</PageContainer>
