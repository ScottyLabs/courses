<script lang="ts">
  // "Send to Stellic" for a degree plan: a script the student runs in their own signed-in Stellic tab that adds the
  // plan's current and future semesters (with synced schedules' sections) to their Stellic plan. Courseweave's
  // features/schedule-tools/SendToStellic.tsx in plan mode.
  import { Check, ClipboardCopy, ExternalLink } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { resolvePlan, type Plan } from "$lib/core/plan";
  import { buildStellicScript, exportPlan, STELLIC_ORIGIN } from "$lib/core/stellic-export";
  import { schedules } from "$lib/state";

  let {
    open = $bindable(false),
    plan,
    current,
    titleOf,
  }: {
    open?: boolean;
    plan: Plan;
    /** The term in progress: past semesters are already in Stellic from the transcript. */
    current: string;
    titleOf: (courseId: string) => string;
  } = $props();

  let copied = $state(false);
  let blocked = $state(false);

  const terms = $derived(
    open ? exportPlan(resolvePlan(plan, schedules.items), schedules.items, current, titleOf) : [],
  );
  const courseCount = $derived(terms.reduce((n, t) => n + t.courses.length, 0));
  const script = $derived(terms.length > 0 ? buildStellicScript(terms) : "");

  async function copy(): Promise<void> {
    try {
      await navigator.clipboard.writeText(script);
      copied = true;
      blocked = false;
      setTimeout(() => (copied = false), 2000);
    } catch {
      blocked = true;
    }
  }
</script>

<Dialog bind:open title="Send plan to Stellic" size="lg">
  <div class="flex flex-col gap-5">
    <section aria-labelledby="send-plan" class="flex flex-col gap-2">
      <h3 id="send-plan" class="text-label-lg text-strong">{plan.name}</h3>
      <p class="text-body-2 text-muted">
        Sends this plan's current and future semesters; past ones are already in Stellic from your transcript.
        Semesters synced with a schedule include its sections. Stellic keeps one main plan, so send one plan at a time.
      </p>
      {#if terms.length > 0}
        <ul class="flex flex-col gap-0.5 text-body-2 text-ink">
          {#each terms as t (t.term)}
            <li><span class="font-semibold text-strong">{t.name}</span>: {t.courses.map((c) => c.code).join(", ")}</li>
          {/each}
        </ul>
      {:else}
        <p class="text-body-2 text-ink">This plan has no courses in this or later semesters yet.</p>
      {/if}
    </section>

    <section aria-labelledby="send-steps" class="flex flex-col gap-3 border-t border-line pt-5">
      <h3 id="send-steps" class="text-heading-6 text-strong">How it works</h3>
      <p class="text-body-2 text-ink">
        Stellic has no import, so this is a short script you run in your own signed-in Stellic tab. It shows you exactly
        what it will add and changes nothing until you confirm. It only adds courses to your plan: it never registers,
        drops or removes anything.
      </p>
      <ol class="flex list-decimal flex-col gap-2 pl-5 text-body-2 text-ink">
        <li>
          <Button size="sm" variant="primary" icon={copied ? Check : ClipboardCopy} disabled={courseCount === 0} onclick={copy}>
            {copied ? "Script copied" : courseCount > 0 ? `Copy script (${courseCount} course${courseCount === 1 ? "" : "s"})` : "Copy script"}
          </Button>
          <span class="sr-only" aria-live="polite">{copied ? "Script copied" : ""}</span>
        </li>
        <li>
          Open
          <a href="{STELLIC_ORIGIN}/" target="_blank" rel="noreferrer" class="inline-flex items-center gap-1 font-semibold text-accent-ink hover:underline">
            Stellic <ExternalLink aria-hidden="true" class="size-3" /><span class="sr-only">(opens in a new tab)</span>
          </a>
          and sign in.
        </li>
        <li>
          Open the console: <kbd class="rounded-tag border border-line px-1 text-label-md">Option+Command+J</kbd> on a Mac or
          <kbd class="rounded-tag border border-line px-1 text-label-md">Ctrl+Shift+J</kbd> in Chrome or Edge. In Safari, turn
          on Show features for web developers in Settings, Advanced, then press
          <kbd class="rounded-tag border border-line px-1 text-label-md">Option+Command+C</kbd>.
        </li>
        <li>Paste the script and press Enter. Chrome may first ask you to type "allow pasting".</li>
        <li>Check the list it shows and confirm. Reload Stellic to see the courses.</li>
      </ol>
      <p class="text-label-md font-normal text-muted">Only paste code into a console from a source you trust. You can read this one first:</p>
      <details class="rounded-tag border border-line">
        <summary class="flex min-h-10 cursor-pointer items-center px-3 text-label-lg text-ink select-none">Show the script</summary>
        <textarea
          readonly
          value={script}
          rows="12"
          aria-label="Stellic script"
          onfocus={(event) => event.currentTarget.select()}
          class="block w-full resize-y border-0 border-t border-line bg-paper p-2 font-mono text-[11.5px] text-ink"
        ></textarea>
      </details>
      {#if blocked}
        <p role="alert" class="text-body-2 text-danger">
          Your browser blocked copying. Open "Show the script", select the text and copy it yourself.
        </p>
      {/if}
    </section>
  </div>
</Dialog>
