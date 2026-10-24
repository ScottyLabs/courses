<script lang="ts">
  // A nudge on pages that depend on the academic record (the degree plan) while it's empty. Hiding it is remembered
  // with the student's preferences. Courseweave's RecordPrompt in features/record/RecordStart.tsx.
  import { PencilLine, X } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { profilePath } from "$lib/core/routes";
  import { preferences, record } from "$lib/state";
  import SioImportButton from "./SioImportButton.svelte";

  let { children }: { children: Snippet } = $props();
</script>

{#if record.isEmpty && !preferences.onboarding.promptHidden}
  <!-- The buttons sit beside the text only when the card itself is wide enough (a container query), so the text
       never squeezes into a narrow column, as it would in the first-run plan's column. -->
  <section
    aria-labelledby="record-prompt-title"
    class="@container relative rounded-card border border-pressed bg-surface p-4 pr-14"
  >
    <div class="flex flex-col gap-3 @3xl:flex-row @3xl:items-center @3xl:gap-5">
      <div class="min-w-0 flex-1">
        <h2 id="record-prompt-title" class="text-heading-6 text-strong">Add courses you've already taken</h2>
        <p class="mt-0.5 text-body-2 text-ink">{@render children()}</p>
      </div>
      <div class="flex flex-wrap gap-2 @3xl:shrink-0">
        <SioImportButton />
        <Button href={profilePath("taken")} icon={PencilLine}>Add by hand</Button>
      </div>
    </div>
    <IconButton
      label="Hide this"
      icon={X}
      size="md"
      class="absolute top-2 right-2"
      onclick={() => preferences.hideOnboardingPrompt()}
    />
  </section>
{/if}
