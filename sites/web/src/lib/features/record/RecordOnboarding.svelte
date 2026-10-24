<script lang="ts">
  // Asks new students (and each new account) to add their academic record, once. The root layout mounts it; it opens
  // when personal.onboardingDue says so, except on Courses Taken, which asks on its own, and stays open once shown so
  // a record imported from it can show its confirmation. Courseweave's RecordOnboarding (features/record/RecordStart.tsx)
  // and the Onboarding wrapper in its App.tsx.
  import { Check, FileText, PencilLine } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import Spinner from "$lib/components/ui/Spinner.svelte";
  import { profilePath } from "$lib/core/routes";
  import { session } from "$lib/data";
  import { personal } from "$lib/state";
  import SioImportButton from "./SioImportButton.svelte";

  const RECORD_PAGE = profilePath("taken");

  let open = $state(false);
  let imported = $state<string | null>(null);

  $effect(() => {
    if (personal.onboardingDue && page.url.pathname !== RECORD_PAGE) open = true;
  });

  const firstName = $derived(session.user?.name.split(/\s+/u)[0]);
  const signedIn = $derived(session.status === "signed-in");
  const title = $derived.by(() => {
    if (imported !== null) return "Record added";
    if (signedIn && firstName !== undefined && firstName !== "") return `Welcome, ${firstName}`;
    return "Welcome to CMU Courses";
  });

  function finish(): void {
    personal.markOnboardingAsked();
    open = false;
  }

  function openRecord(): void {
    finish();
    void goto(RECORD_PAGE);
  }
</script>

<Dialog bind:open {title} size="md" onclose={() => personal.markOnboardingAsked()}>
  {#if imported !== null}
    <div class="flex flex-col gap-3">
      <p role="status" class="flex items-start gap-2 text-body-1 font-medium text-strong">
        <Check aria-hidden="true" class="mt-0.5 size-5 shrink-0 text-ok" />
        {imported}
      </p>
      <p class="text-body-2 text-ink">
        Prerequisite checks and your degree plan now count it. Placements, waivers and anything else SIO doesn't list
        can be added under Courses Taken in your profile.
      </p>
    </div>
  {:else}
    <div class="flex flex-col gap-4">
      <div>
        <h3 class="text-heading-5 text-strong">Start with what you've already taken</h3>
        <p class="mt-1.5 text-body-2 text-ink">
          CMU Courses checks prerequisites and degree requirements against what you've earned: CMU courses, AP and IB
          scores (incoming students too), transfer credit and placements.
        </p>
      </div>
      <div class="flex flex-col gap-2.5">
        <SioImportButton
          onimported={(message) => {
            personal.markOnboardingAsked();
            imported = message;
          }}
        >
          {#snippet trigger({ choose, reading })}
            <button
              type="button"
              onclick={choose}
              disabled={reading}
              aria-describedby="onboarding-import-desc"
              class="flex w-full items-start gap-3 rounded-card border border-accent bg-surface p-4 text-left transition-colors hover:bg-hover disabled:opacity-60"
            >
              <span class="mt-0.5 grid size-10 shrink-0 place-items-center rounded-field bg-accent text-on-accent">
                {#if reading}
                  <Spinner size="sm" label="Reading the PDF" />
                {:else}
                  <FileText aria-hidden="true" class="size-5" />
                {/if}
              </span>
              <span class="flex min-w-0 flex-col gap-1">
                <span class="text-label-lg text-strong">Import from SIO or Stellic</span>
                <span id="onboarding-import-desc" class="flex flex-col gap-1">
                  <span class="text-body-2 text-ink">
                    Add courses and credit from your SIO academic record PDF or your Stellic plan PDF, which also
                    becomes a degree plan. You pick what to bring in.
                  </span>
                  <span class="text-label-md font-normal text-muted">
                    SIO: Academic Record, then PDF under Unofficial Academic Record. Stellic: Plan your Path, then
                    Download Plan. It's read on this device, never uploaded.
                  </span>
                </span>
              </span>
            </button>
          {/snippet}
        </SioImportButton>
        <button
          type="button"
          onclick={openRecord}
          aria-describedby="onboarding-hand-desc"
          class="flex w-full items-start gap-3 rounded-card border border-line-strong bg-surface p-4 text-left transition-colors hover:bg-hover"
        >
          <span class="mt-0.5 grid size-10 shrink-0 place-items-center rounded-field bg-paper text-ink">
            <PencilLine aria-hidden="true" class="size-5" />
          </span>
          <span class="flex min-w-0 flex-col gap-1">
            <span class="text-label-lg text-strong">Enter it by hand</span>
            <span id="onboarding-hand-desc" class="text-body-2 text-ink">
              Add courses, AP or IB scores and transfer credit yourself under Courses Taken in your profile.
            </span>
          </span>
        </button>
      </div>
      {#if !signedIn}
        <p class="text-body-2 text-muted">
          You're not signed in, so what you add stays in this tab until you sign in with your Andrew ID.
        </p>
      {/if}
    </div>
  {/if}
  {#snippet footer()}
    {#if imported !== null}
      <Button variant="ghost" onclick={finish}>Done</Button>
      <Button variant="primary" onclick={openRecord}>Open my record</Button>
    {:else}
      <p class="mr-auto self-center text-body-2 text-muted">Nothing to add yet? You can do this any time.</p>
      <Button variant="ghost" onclick={finish}>Skip for now</Button>
    {/if}
  {/snippet}
</Dialog>
