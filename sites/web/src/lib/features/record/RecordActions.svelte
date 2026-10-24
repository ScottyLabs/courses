<script lang="ts">
  // The record's actions: import from SIO or Stellic (the empty record's panel offers it instead), back up to a JSON
  // file and restore one (merging or replacing), and clear the record. Courseweave's Header in
  // features/record/RecordPage.tsx.
  import { Download, Trash2, Upload } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
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

  function download(): void {
    const data = exportRecord(record.data);
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `cmucourses-record-${new Date().toISOString().slice(0, 10)}.json`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
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

</script>

<div class="flex flex-wrap items-center gap-2">
  {#if !record.isEmpty}
    <SioImportButton variant="primary" onimported={(message) => toast(message, { kind: "success" })} />
  {/if}
  <Button icon={Download} disabled={empty} onclick={download}>Export backup</Button>
  <Button icon={Upload} onclick={() => fileInput?.click()}>Import backup</Button>
  {#if !empty}
    <Button variant="danger" icon={Trash2} onclick={() => (confirmClear = true)}>Clear record</Button>
  {/if}
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
  <p role="alert" class="mt-2 text-body-2 text-danger">{error}</p>
{/if}

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
