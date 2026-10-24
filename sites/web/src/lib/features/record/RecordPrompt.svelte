<script lang="ts">
  // A nudge on pages that depend on the academic record (the degree plan) while it's empty; can be hidden.
  // Courseweave's RecordPrompt in features/record/RecordStart.tsx. (Placeholder until the record feature lands.)
  import type { Snippet } from "svelte";
  import { personal, record } from "$lib/state";

  let { children }: { children: Snippet } = $props();
</script>

{#if record.isEmpty && !personal.preferences.onboarding.promptHidden}
  <section aria-label="Add your academic record" class="rounded-card border border-pressed bg-surface p-4">
    <h2 class="text-heading-6 text-strong">Add courses you've already taken</h2>
    <p class="mt-1 text-body-2 text-ink">{@render children()}</p>
  </section>
{/if}
