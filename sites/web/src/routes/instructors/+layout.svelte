<script lang="ts">
  import type { Snippet } from "svelte";
  import InstructorsShell from "$lib/features/instructors/InstructorsShell.svelte";

  let { children }: { children: Snippet } = $props();
</script>

<InstructorsShell>{@render children()}</InstructorsShell>
