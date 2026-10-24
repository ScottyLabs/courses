<script lang="ts" module>
  import type { CourseInfo } from "$lib/core/record/hooks";
  import type { ImportPlan } from "$lib/core/record/SioImportPlan";
  import type { DegreePlanDraft } from "$lib/core/record/StellicImportPlan";

  export interface ImportReviewState {
    source: "sio" | "stellic";
    intro: string;
    /** Lines that couldn't be read or weren't imported. */
    warnings: string[];
    plan: ImportPlan;
    /** A Stellic plan's semesters, offered as a new degree plan. */
    draft: DegreePlanDraft | null;
    lookup: (courseId: string) => CourseInfo;
  }
</script>

<script lang="ts">
  // "Import from SIO or Stellic": reads an SIO unofficial academic record PDF, or a Stellic plan PDF, in the browser
  // and lets the student choose what to add to the record. A Stellic plan also becomes a degree plan. The file never
  // leaves the device (decision 17): pdf.js and the parsers load only when a file is chosen. Courseweave's
  // features/record/SioImport.tsx.
  import { FileText, Upload } from "@lucide/svelte";
  import type { Snippet } from "svelte";
  import Button, { type ButtonSize, type ButtonVariant } from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { currentTerm } from "$lib/core/degree/hooks";
  import { courseLookup } from "$lib/core/record/hooks";
  import { planSioImport } from "$lib/core/record/SioImportPlan";
  import { planStellicImport } from "$lib/core/record/StellicImportPlan";
  import { NotSioRecordError } from "$lib/core/sio-record";
  import { COLLEGES } from "$lib/core/state/record";
  import { termLabel } from "$lib/core/requisites";
  import { catalog } from "$lib/data";
  import { plans, record } from "$lib/state";
  import HowTo from "./HowTo.svelte";
  import ImportReview from "./ImportReview.svelte";

  let {
    onimported,
    onplancreated,
    variant = "outline",
    size = "md",
    label = "Import from SIO or Stellic",
    hidden = false,
    trigger,
  }: {
    /** After items were added to the record or a plan was created, with a sentence saying what happened. Without it, a toast says so. */
    onimported?: (message: string) => void;
    /** After a Stellic plan became a new degree plan. */
    onplancreated?: (planId: string) => void;
    variant?: ButtonVariant;
    size?: ButtonSize;
    label?: string;
    /** Render no button; open the file chooser with `choose()` (from a menu item). */
    hidden?: boolean;
    /** A custom trigger in place of the default button. */
    trigger?: Snippet<[{ choose: () => void; reading: boolean }]>;
  } = $props();

  type State =
    | { kind: "closed" }
    | { kind: "reading" }
    | { kind: "error"; message: string }
    | ({ kind: "review" } & ImportReviewState);

  const FROM = { sio: "your SIO record", stellic: "your Stellic plan" } as const;
  /** pdf.js names for errors caused by the file itself (corrupt, encrypted, empty), not by the browser. */
  const BAD_FILE_ERRORS = new Set(["InvalidPDFException", "PasswordException", "FormatError"]);

  let input = $state<HTMLInputElement>();
  let current = $state<State>({ kind: "closed" });
  let open = $state(false);
  let checked = $state<Set<string>>(new Set());
  let useProfile = $state(true);
  let useDraft = $state(true);

  const reading = $derived(current.kind === "reading");

  /** Opens the file chooser. */
  export function choose(): void {
    input?.click();
  }

  function errorMessage(error: unknown): string {
    if (error instanceof NotSioRecordError) return error.message;
    if (error instanceof Error && BAD_FILE_ERRORS.has(error.name))
      return "That PDF couldn't be read. Download it again and retry.";
    return "This browser couldn't read the PDF. Update it, or try again in another browser such as Chrome or Firefox.";
  }

  async function review(file: File): Promise<ImportReviewState> {
    // pdf.js and the catalog load together; the catalog names courses and gives units.
    const [{ readImportPdf }, index] = await Promise.all([
      import("$lib/core/record/SioImportPdf"),
      catalog.catalogIndex().catch(() => null),
    ]);
    const pdf = await readImportPdf(file);
    const lookup = courseLookup(null, index);
    const profile = record.profile;
    if (pdf.kind === "sio") {
      const plan = planSioImport(pdf.record, record.entries, {
        knows: (id) => Boolean(lookup(id).title),
        currentTerm: currentTerm(),
        profile: { college: profile.college, entryTerm: profile.entryTerm },
      });
      const asOf =
        pdf.record.asOf === null || pdf.record.asOf === ""
          ? ""
          : `Your unofficial academic record as of ${pdf.record.asOf}. `;
      return {
        source: "sio",
        intro: `${asOf}Choose what to add to your record. Items already in it start unchecked.`,
        warnings: pdf.record.warnings,
        plan,
        draft: null,
        lookup,
      };
    }
    // Program versions are member data: signed out, programs just aren't matched.
    const programs = await catalog.programsIndex().catch(() => null);
    const imported = planStellicImport(pdf.plan, record.entries, {
      knows: (id) => Boolean(lookup(id).title),
      catalogUnits: (id) => lookup(id).units,
      programs,
      existingPlanNames: plans.list.map((p) => p.name),
      currentTerm: currentTerm(),
      profile: { entryTerm: profile.entryTerm },
    });
    return {
      source: "stellic",
      intro:
        "Your Stellic plan becomes a new degree plan. Credit and courses Stellic marks as taken can go in your record too; items already in it start unchecked.",
      warnings: imported.skipped,
      plan: imported,
      draft: imported.draft,
      lookup,
    };
  }

  async function onFile(file: File): Promise<void> {
    current = { kind: "reading" };
    try {
      const next = await review(file);
      checked = new Set(
        next.plan.groups.flatMap((g) => g.items.filter((i) => !i.already).map((i) => i.key)),
      );
      useProfile = true;
      useDraft = true;
      current = { kind: "review", ...next };
    } catch (error) {
      console.error("PDF import failed", error);
      current = { kind: "error", message: errorMessage(error) };
    }
    open = true;
  }

  function onchange(): void {
    const file = input?.files?.[0];
    if (input) input.value = "";
    if (file !== undefined) void onFile(file);
  }

  function close(): void {
    open = false;
  }

  function onclosed(): void {
    if (current.kind !== "reading") current = { kind: "closed" };
  }

  const profileText = $derived.by(() => {
    if (current.kind !== "review") return "";
    const p = current.plan.profile;
    return [
      p.college === undefined
        ? null
        : `college to ${COLLEGES.find((c) => c.code === p.college)?.name ?? p.college}`,
      p.entryTerm === undefined ? null : `first semester to ${termLabel(p.entryTerm)}`,
    ]
      .filter((x) => x !== null)
      .join(" and ");
  });

  const total = $derived(
    current.kind === "review"
      ? current.plan.groups.reduce((n, g) => n + g.items.filter((i) => checked.has(i.key)).length, 0)
      : 0,
  );
  const creating = $derived(current.kind === "review" && current.draft !== null && useDraft);
  const canImport = $derived(total > 0 || creating || (useProfile && profileText !== ""));
  const importLabel = $derived.by(() => {
    const items = `${total} ${total === 1 ? "item" : "items"}`;
    if (creating) return total > 0 ? `Create plan and import ${total}` : "Create plan";
    return total > 0 ? `Import ${items}` : "Update profile";
  });

  function doImport(): void {
    if (current.kind !== "review") return;
    const chosen = current.plan.groups.flatMap((g) => g.items).filter((i) => checked.has(i.key));
    if (chosen.length > 0) record.addEntries(chosen.map((i) => i.entry));
    const patch = useProfile ? current.plan.profile : {};
    if (patch.college !== undefined || patch.entryTerm !== undefined) record.setProfile(patch);
    const draft = useDraft ? current.draft : null;
    const planId =
      draft === null
        ? null
        : plans.createPlan({
            name: draft.name,
            startTerm: draft.startTerm,
            programs: draft.programs,
            terms: draft.terms,
          });
    const items = `${chosen.length} ${chosen.length === 1 ? "item" : "items"}`;
    let message: string;
    if (draft !== null)
      message = `Created the degree plan \u201c${draft.name}\u201d${chosen.length > 0 ? ` and added ${items} to your record` : ""}.`;
    else if (chosen.length > 0) message = `Imported ${items} from ${FROM[current.source]}.`;
    else message = `Updated your profile from ${FROM[current.source]}.`;
    close();
    if (onimported) onimported(message);
    else toast(message, { kind: "success" });
    if (planId !== null) onplancreated?.(planId);
  }

  const title = $derived(
    current.kind === "review" && current.source === "stellic" ? "Import from Stellic" : "Import a PDF",
  );
</script>

{#if trigger}
  {@render trigger({ choose, reading })}
{:else if !hidden}
  <Button {variant} {size} icon={FileText} loading={reading} onclick={choose}>{label}</Button>
{/if}
<input
  bind:this={input}
  type="file"
  accept="application/pdf,.pdf"
  class="hidden"
  tabindex="-1"
  aria-hidden="true"
  {onchange}
/>
<p class="sr-only" aria-live="polite">{reading ? "Reading the PDF" : ""}</p>

<Dialog bind:open {title} size="lg" onclose={onclosed}>
  {#if current.kind === "error"}
    <div class="flex flex-col gap-4">
      <p role="alert" class="text-body-1 text-danger">{current.message}</p>
      <HowTo />
    </div>
  {:else if current.kind === "review"}
    <ImportReview
      review={current}
      bind:checked
      bind:useProfile
      bind:useDraft
      {profileText}
    />
  {/if}
  {#snippet footer()}
    {#if current.kind === "error"}
      <Button variant="ghost" onclick={close}>Close</Button>
      <Button
        variant="primary"
        icon={Upload}
        onclick={() => {
          close();
          choose();
        }}
      >
        Choose another file
      </Button>
    {:else if current.kind === "review"}
      <p class="mr-auto self-center text-body-2 text-muted" aria-live="polite">
        {total}
        {total === 1 ? "item" : "items"} selected
      </p>
      <Button variant="ghost" onclick={close}>Cancel</Button>
      <Button variant="primary" disabled={!canImport} onclick={doImport}>{importLabel}</Button>
    {/if}
  {/snippet}
</Dialog>
