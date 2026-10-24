<script lang="ts">
  import { Heart, MessageSquareText } from "@lucide/svelte";
  import Button from "../ui/Button.svelte";

  const FEEDBACK_URL = "https://git.cmu.dev/ScottyLabs/courses/issues";
  const REPOSITORY_URL = "https://git.cmu.dev/ScottyLabs/courses";
  const SCOTTYLABS_URL = "https://scottylabs.org";
</script>

<footer class="border-t border-line bg-paper">
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
    <Button href={FEEDBACK_URL} icon={MessageSquareText} class="shrink-0">Feedback</Button>
  </div>
</footer>
