<script lang="ts">
  // Rules the audit doesn't check (GPA, approvals), behind a disclosure.
  import { Info } from "@lucide/svelte";

  let { notes }: { notes: string[] } = $props();
</script>

<details class="mt-1.5 text-label-md text-muted">
  <summary class="inline-flex min-h-10 cursor-pointer items-center gap-1 select-none hover:text-ink md:min-h-6">
    <Info aria-hidden="true" class="size-3.5" />
    {notes.length === 1 ? "1 rule not checked here" : `${notes.length} rules not checked here`}
  </summary>
  <ul class="mt-1 flex list-disc flex-col gap-0.5 pl-5 font-normal">
    {#each notes as n (n)}
      <li class="whitespace-pre-line">{n}</li>
    {/each}
  </ul>
</details>
