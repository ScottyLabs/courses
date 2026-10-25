<script lang="ts">
  import { ArrowRight } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import type { OtherMatch } from "./results";
  import { nearestTerm } from "$lib/core/offerings";
  import { codeOf } from "$lib/core/requisite-format";
  import { termLabel } from "$lib/core/requisites";
  import { coursePath } from "$lib/core/routes";

  let {
    heading,
    intro,
    note,
    matches,
    total = matches.length,
    termId,
    termSpan = null,
    busy = false,
    more,
  }: {
    heading: string;
    /** Shown above the heading, such as "No Spring 2027 courses match". */
    intro?: string;
    /** Under the heading. */
    note?: string;
    matches: readonly OtherMatch[];
    /** All matches, when only some are shown. */
    total?: number;
    /** The semester being browsed; links suggest the offering closest to it. */
    termId: string;
    /** "Fall 2025-Spring 2027": the built semesters, for courses none of them offer. */
    termSpan?: string | null;
    busy?: boolean;
    /** Shown after the list, such as a "Show more" button. */
    more?: Snippet;
  } = $props();
</script>

<section aria-label={heading} aria-busy={busy} class="mt-6">
  {#if intro}<p class="mb-2 text-body-1 text-ink">{intro}</p>{/if}
  <h2 class="text-heading-6 text-strong">
    {heading}
    {#if total > 0}<span class="tabular ml-1 text-label-md text-muted">{total.toLocaleString("en-US")}</span>{/if}
  </h2>
  {#if note}<p class="mt-0.5 text-body-2 text-muted">{note}</p>{/if}
  {#if matches.length > 0}
    <ul class="mt-3 overflow-hidden rounded-card border border-pressed bg-surface">
      {#each matches as match (match.id)}
        {@const next = nearestTerm([...match.terms], termId)}
        <li class="flex flex-wrap items-center gap-x-3 gap-y-1 border-b border-line px-4 py-2.5 last:border-b-0">
          <span class="tabular code-column text-heading-6 text-muted">{codeOf(match.id)}</span>
          <div class="min-w-0 flex-1">
            <a href={coursePath(match.id, next ?? null)} class="block truncate text-body-1 text-ink hover:underline"
              >{match.title}</a
            >
            <p class="text-label-md text-muted">
              {#if match.terms.length === 0}
                Not offered in {termSpan ? `any semester here (${termSpan})` : "a recent semester"}
              {:else}
                Offered in
                {#each match.terms as term, index (term)}{index > 0 ? ", " : ""}<a
                    href={coursePath(match.id, term)}
                    class="whitespace-nowrap underline decoration-line-strong underline-offset-2 hover:text-ink"
                    >{termLabel(term)}</a
                  >{/each}
              {/if}
            </p>
          </div>
          {#if next}
            <a
              href={coursePath(match.id, next)}
              class="inline-flex h-10 shrink-0 items-center gap-1.5 rounded-full border border-line-strong px-3.5 text-label-md text-ink hover:bg-hover"
            >
              View in {termLabel(next)}<ArrowRight aria-hidden="true" class="size-4" />
            </a>
          {:else}
            <a
              href={coursePath(match.id)}
              class="inline-flex h-10 shrink-0 items-center gap-1.5 rounded-full border border-line-strong px-3.5 text-label-md text-ink hover:bg-hover"
            >
              Catalog page<ArrowRight aria-hidden="true" class="size-4" />
            </a>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  {@render more?.()}
</section>
