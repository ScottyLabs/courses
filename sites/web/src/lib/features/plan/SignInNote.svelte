<script lang="ts">
  // Where degree programs would be for a visitor who isn't signed in (or isn't a CMU account): what they need and a
  // sign-in button. Degree programs are member data.
  import { LogIn } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { session } from "$lib/data";

  let { title, forbidden = false }: { title: string; forbidden?: boolean } = $props();
</script>

<div class="flex flex-col gap-3 rounded-card border border-dashed border-line-strong bg-surface p-4 sm:flex-row sm:items-center">
  <span class="grid size-10 shrink-0 place-items-center rounded-full bg-paper text-ink">
    <LogIn aria-hidden="true" class="size-5" />
  </span>
  <div class="min-w-0 flex-1">
    <p class="text-label-lg text-strong">{forbidden ? "Degree programs need a CMU account" : title}</p>
    <p class="mt-0.5 text-body-2 text-ink">
      Majors, minors and their requirements come from CMU's degree audit, so they're only for signed-in CMU students.
      Your semesters work without them.
    </p>
  </div>
  {#if !forbidden && session.status !== "loading"}
    <Button variant="primary" size="sm" icon={LogIn} class="max-md:h-10" onclick={() => session.signIn()}>Sign in</Button>
  {/if}
</div>
