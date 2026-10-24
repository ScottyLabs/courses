<script lang="ts">
  // The record's actions, beside the Courses Taken title: import from SIO or Stellic (the empty record's panel offers
  // it instead), and under More, back up to a JSON file, restore one (merging or replacing) and clear the record.
  // Courseweave's Header in features/record/RecordPage.tsx.
  import { ChevronDown, Download, Ellipsis, Trash2, Upload } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import { download } from "$lib/components/ui/download";
  import Menu, { type MenuEntry } from "$lib/components/ui/Menu.svelte";
  import { toast } from "$lib/components/ui/toast.svelte";
  import { exportRecord, parseRecordExport, type RecordData } from "$lib/core/state/record";
  import { record } from "$lib/state";
  import SioImportButton from "./SioImportButton.svelte";

  let fileInput = $state<HTMLInputElement>();
  let pending = $state<RecordData | null>(null);
  let pendingOpen = $state(false);
  let confirmClear = $state(false);
  let error = $state<string | null>(null);

  const empty = $derived(record.isEmpty && Object.keys(record.profile).length === 0);

  function exportBackup(): void {
    const data = exportRecord(record.data);
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
    download(blob, `cmucourses-record-${new Date().toISOString().slice(0, 10)}.json`);
  }

  async function onFile(file: File): Promise<void> {
    error = null;
    let parsed: ReturnType<typeof parseRecordExport>;
    try {
      parsed = parseRecordExport(JSON.parse(await file.text()));
    } catch {
      error = "That file couldn't be read as JSON. Choose a record backup exported from CMU Courses or Courseweave.";
      return;
    }
    if ("error" in parsed) {
      error = parsed.error;
      return;
    }
    if (record.isEmpty) {
      record.importRecord(parsed.data, "replace");
      toast(`Imported ${parsed.data.entries.length} entries.`, { kind: "success" });
      return;
    }
    pending = parsed.data;
    pendingOpen = true;
  }

  function restore(mode: "merge" | "replace"): void {
    if (pending !== null) record.importRecord(pending, mode);
    toast(
      mode === "merge" ? "Merged the backup into your record." : "Replaced your record with the backup.",
      { kind: "success" },
    );
    pendingOpen = false;
  }

  const moreItems = $derived<MenuEntry[]>([
    { label: "Export backup", icon: Download, disabled: empty, onselect: exportBackup },
    { label: "Import backup", icon: Upload, onselect: () => fileInput?.click() },
    ...(empty
      ? []
      : [
          "separator" as const,
          { label: "Clear record", icon: Trash2, danger: true, onselect: () => (confirmClear = true) },
        ]),
  ]);
</script>

<div class="flex flex-col gap-2 sm:items-end">
<div class="flex flex-wrap items-center gap-2">
  {#if !record.isEmpty}
    <SioImportButton variant="primary" onimported={(message) => toast(message, { kind: "success" })} />
  {/if}
  <Menu items={moreItems} label="More record actions">
    {#snippet trigger(props)}
      <Button
        {...props}
        icon={Ellipsis}
        aria-label="More record actions"
        title="Back up, restore or clear your record"
        class="max-sm:w-10 max-sm:px-0"
      >
        <span class="max-sm:sr-only">More</span>
        <ChevronDown aria-hidden="true" class="max-sm:hidden" />
      </Button>
    {/snippet}
  </Menu>
  <input
    bind:this={fileInput}
    type="file"
    accept="application/json,.json"
    class="hidden"
    tabindex="-1"
    aria-hidden="true"
    onchange={(event) => {
      const file = event.currentTarget.files?.[0];
      event.currentTarget.value = "";
      if (file !== undefined) void onFile(file);
    }}
  />
</div>
{#if error !== null}
  <p role="alert" class="max-w-[48ch] text-body-2 text-danger">{error}</p>
{/if}
</div>

<Dialog bind:open={pendingOpen} title="Import backup" size="sm" onclose={() => (pending = null)}>
  <p class="text-body-2 text-ink">
    The file has {pending?.entries.length ?? 0}
    {pending?.entries.length === 1 ? "entry" : "entries"} and {Object.keys(pending?.overrides ?? {}).length} overrides.
    Add them to your current record, or replace it entirely?
  </p>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (pendingOpen = false)}>Cancel</Button>
    <Button onclick={() => restore("replace")}>Replace</Button>
    <Button variant="primary" onclick={() => restore("merge")}>Merge</Button>
  {/snippet}
</Dialog>

<Dialog bind:open={confirmClear} title="Clear record" size="sm">
  <p class="text-body-2 text-ink">
    This removes every course, credit, placement and override from your record. Export a backup first if you might want
    it back.
  </p>
  {#snippet footer()}
    <Button variant="ghost" onclick={() => (confirmClear = false)}>Cancel</Button>
    <Button
      variant="danger"
      onclick={() => {
        record.clearRecord();
        confirmClear = false;
        toast("Cleared your record.");
      }}
    >
      Clear record
    </Button>
  {/snippet}
</Dialog>
