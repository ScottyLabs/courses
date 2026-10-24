<script lang="ts" module>
  import type { RequisiteStatus } from "$lib/components/ui/StatusChip.svelte";
  import type { CatalogIndex } from "$lib/core/catalog-types";

  /** What every node of a requisite tree reads. */
  export interface TreeContext {
    index: CatalogIndex;
    href: (courseId: string) => string;
    title: (courseId: string) => string | undefined;
    status?: (courseId: string) => RequisiteStatus | undefined;
    expandAll: boolean;
  }

  /** Deepest level "Expand all" opens; deeper chains stay collapsed to keep the tree readable. */
  export const MAX_AUTO_DEPTH = 6;
</script>

<script lang="ts">
  import { ChevronRight } from "@lucide/svelte";
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import type { Requisite } from "$lib/core/catalog-types";
  import { codeOf } from "$lib/core/requisite-format";
  import Self from "./RequisiteTreeNode.svelte";

  let {
    req,
    ctx,
    path,
    depth,
  }: {
    req: Requisite;
    ctx: TreeContext;
    /** Course ids from the root down to here, to stop at cycles. */
    path: readonly string[];
    depth: number;
  } = $props();

  const groupLabel = $derived(
    req.type === "all"
      ? "All of"
      : req.type === "any"
        ? "One of"
        : req.type === "atLeast"
          ? `At least ${req.n} of`
          : "",
  );
  const id = $derived(req.type === "course" ? req.id : "");
  const prereqs = $derived(req.type === "course" ? (ctx.index.courses[req.id]?.prereqs ?? null) : null);
  const cycle = $derived(req.type === "course" && path.includes(req.id));
  const expandable = $derived(prereqs !== null && !cycle);
  // svelte-ignore state_referenced_locally
  let open = $state(ctx.expandAll && depth < MAX_AUTO_DEPTH);
</script>

{#if req.type === "course"}
  <li class="py-0.5">
    <div class="flex min-w-0 items-center gap-1.5">
      {#if expandable}
        <button
          type="button"
          aria-expanded={open}
          aria-label="{open ? 'Hide' : 'Show'} prerequisites of {codeOf(id)}"
          onclick={() => (open = !open)}
          class="grid size-10 shrink-0 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink sm:size-8"
        >
          <ChevronRight aria-hidden="true" class="size-4 transition-transform {open ? 'rotate-90' : ''}" />
        </button>
      {:else}
        <span aria-hidden="true" class="size-10 shrink-0 sm:size-8"></span>
      {/if}
      <CourseChip code={codeOf(id)} href={ctx.href(id)} status={ctx.status?.(id)} />
      {#if req.minGrade}<span class="shrink-0 text-label-md text-muted">min {req.minGrade}</span>{/if}
      <span class="min-w-0 truncate text-body-2 text-ink">{ctx.title(id) ?? ""}</span>
      {#if cycle}<span class="shrink-0 text-label-md text-muted">(circular)</span>{/if}
    </div>
    {#if open && prereqs}
      <ul class="ml-5 border-l border-line pl-2 sm:ml-4 sm:pl-3">
        <Self req={prereqs} {ctx} path={[...path, id]} depth={depth + 1} />
      </ul>
    {/if}
  </li>
{:else if req.type === "text"}
  <li class="py-1 pl-10 text-body-2 text-ink italic sm:pl-8">{req.text}</li>
{:else}
  <li class="py-0.5">
    <span class="text-label-md text-muted">{groupLabel}</span>
    <ul class="mt-0.5 ml-1.5 border-l border-line-strong pl-2.5 sm:ml-2 sm:pl-3">
      {#each req.items as item, index (index)}
        <Self req={item} {ctx} {path} {depth} />
      {/each}
    </ul>
  </li>
{/if}
