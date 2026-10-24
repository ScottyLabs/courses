<script lang="ts">
  // A semester's footer: factorable and pass/fail units, FCE hours a week (at the busier half when minis run), and
  // load guidance.
  import type { TermTotals } from "$lib/core/plan";

  let { totals, warnings }: { totals: TermTotals; warnings: string[] } = $props();

  const fceTitle = $derived(
    (totals.peakHalf
      ? `Busier half of the term (${totals.peakHalf} half): ${Math.round(totals.peakHours)} FCE hours a week, against ${Math.round(totals.otherHalfHours)} in the other half. Full-semester courses count in both halves; minis only in theirs, at their in-session hours. Units there suggest about ${Math.round(totals.peakExpected)} hours a week (mini units count double). Which half a mini runs in is estimated from past FCEs.`
      : `FCE hours a week for the ${totals.fceUnits} units with evaluations.`) +
      (totals.missingFce > 0
        ? ` ${totals.missingFce} course${totals.missingFce === 1 ? " has" : "s have"} no FCEs.`
        : ""),
  );
</script>

<footer class="flex flex-col gap-1 border-t border-line px-4 py-2 text-label-md font-normal">
  <div class="flex flex-wrap items-baseline gap-x-3 gap-y-0.5 text-ink">
    <span title="Factorable units count toward your QPA">
      <span class="tabular font-semibold text-strong">{totals.factorable}</span> factorable
    </span>
    {#if totals.nonFactorable > 0}
      <span title="Pass/fail units count toward graduation but not your QPA">
        <span class="tabular font-semibold text-strong">{totals.nonFactorable}</span> pass/fail
      </span>
    {/if}
    {#if totals.fceUnits > 0}
      <span class="ml-auto" title={fceTitle}>
        <span class="tabular font-semibold text-strong">{Math.round(totals.peakHours)}</span>
        FCE hrs/wk{totals.peakHalf ? " at peak" : ""}
        <span class="sr-only">. {fceTitle}</span>
      </span>
    {/if}
  </div>
  {#each warnings as w (w)}
    <p class="text-warn">{w}. <span class="text-muted">Guidance, not policy.</span></p>
  {/each}
</footer>
