<script lang="ts">
  // A requisite expression as one line of wrapping text, the way a catalog writes it: "15-122 and (21-127 or
  // 15-151)". Courses link to their pages and are highlighted done, planned or missing. For the detailed layout's
  // Prerequisites column; Courseweave's features/catalog/RequisiteText.tsx.
  import type { Requisite } from "$lib/core/catalog-types";
  import type { RequisiteStatus, StatusFn } from "$lib/core/catalog/chip-context";
  import { codeOf } from "$lib/core/requisite-format";

  let {
    req,
    status,
    href,
    title,
    class: className = "",
  }: {
    req: Requisite;
    status?: StatusFn;
    href: (courseId: string) => string;
    title?: (courseId: string) => string | undefined;
    class?: string;
  } = $props();

  const STATUS_LABEL: Record<RequisiteStatus, string> = {
    met: "completed",
    planned: "planned",
    missing: "not taken yet",
  };
  const HIGHLIGHT: Record<RequisiteStatus, string> = {
    met: "bg-ok-bg text-ok",
    planned: "bg-warn-bg text-warn",
    missing: "bg-danger-bg text-danger",
  };

  function label(id: string, minGrade: string | undefined, s: RequisiteStatus | undefined): string {
    const name = title?.(id);
    return [
      name === undefined ? codeOf(id) : `${codeOf(id)} ${name}`,
      minGrade === undefined ? null : `minimum grade ${minGrade}`,
      s === undefined ? null : STATUS_LABEL[s],
    ]
      .filter((x) => x !== null)
      .join(", ");
  }

  const WORD = { all: " and ", any: " or ", atLeast: ", " } as const;
</script>

{#snippet part(r: Requisite, top: boolean)}
  {#if r.type === "course"}
    {@const s = status?.(r.id)}
    <a
      href={href(r.id)}
      aria-label={label(r.id, r.minGrade, s)}
      class="tabular rounded-[3px] whitespace-nowrap hover:underline {s === undefined
        ? 'text-strong hover:text-accent-ink'
        : `px-1 py-px ${HIGHLIGHT[s]}`}">{codeOf(r.id)}</a
    >
  {:else if r.type === "text"}
    <span class="italic">{r.text}</span>
  {:else if r.items.length === 1 && r.items[0]}
    {@render part(r.items[0], top)}
  {:else}
    {#if r.type === "atLeast"}<span class="text-muted">at least {r.n} of </span>{/if}{#if !top}<span
        class="text-muted">(</span
      >{/if}{#each r.items as item, i (i)}{#if i > 0}<span class="text-muted">{WORD[r.type]}</span>{/if}{@render part(
        item,
        false,
      )}{/each}{#if !top}<span class="text-muted">)</span>{/if}
  {/if}
{/snippet}

<span class="text-label-md leading-[18px] font-normal text-ink {className}">{@render part(req, true)}</span>
