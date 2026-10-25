<script lang="ts">
  // The site footer. It folds to a one-line bar, remembered on the device, for students who'd rather have the room.
  import { ChevronDown, ChevronUp, Heart, MessageSquareText } from "@lucide/svelte";
  import Button from "../ui/Button.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import { readLocal, writeLocal } from "../ui/storage";

  const FEEDBACK_URL = "https://git.cmu.dev/ScottyLabs/courses/issues";
  const REPOSITORY_URL = "https://git.cmu.dev/ScottyLabs/courses";
  const SCOTTYLABS_URL = "https://scottylabs.org";
  const KEY = "footer-collapsed";

  let collapsed = $state(readLocal(KEY) === "1");

  function toggle(): void {
    collapsed = !collapsed;
    writeLocal(KEY, collapsed ? "1" : null);
  }
</script>

<footer class="border-t border-line bg-paper">
  {#if collapsed}
    <div class="flex h-10 items-center justify-between gap-4 px-4 text-label-md font-normal text-muted sm:px-[34px]">
      <p>CMU Courses by <a href={SCOTTYLABS_URL} class="hover:text-ink hover:underline">ScottyLabs</a></p>
      <IconButton label="Show footer" icon={ChevronUp} size="sm" tooltip onclick={toggle} />
    </div>
  {:else}
    <div
      class="flex min-h-[81px] flex-col items-start gap-4 px-4 py-5 sm:flex-row sm:items-center sm:justify-between sm:px-[34px] sm:py-4"
    >
      <div class="text-body-2 text-muted">
        <p>
          Designed, developed and maintained with
          <Heart aria-hidden="true" class="mx-0.5 inline size-[18px] -translate-y-px fill-current text-danger" /><span
            class="sr-only">love</span
          >
          by <a href={SCOTTYLABS_URL} class="hover:text-ink hover:underline">ScottyLabs</a>.
        </p>
        <p>
          Want to help us make CMU Courses better? Join us at
          <a href={SCOTTYLABS_URL} class="hover:text-ink hover:underline">ScottyLabs</a>
          or make a pull request on our
          <a href={REPOSITORY_URL} class="hover:text-ink hover:underline">repository</a>.
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <Button href={FEEDBACK_URL} icon={MessageSquareText}>Feedback</Button>
        <IconButton label="Hide footer" icon={ChevronDown} tooltip onclick={toggle} />
      </div>
    </div>
  {/if}
</footer>
