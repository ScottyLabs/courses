<!-- Renames a schedule. -->
<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Dialog from "$lib/components/ui/Dialog.svelte";
  import TextInput from "$lib/components/ui/TextInput.svelte";

  let {
    open = $bindable(false),
    name,
    onsave,
  }: { open?: boolean; name: string; onsave: (name: string) => void } = $props();

  let value = $state("");
  $effect(() => {
    if (open) value = name;
  });
  const trimmed = $derived(value.trim());

  function save(event?: SubmitEvent): void {
    event?.preventDefault();
    if (trimmed === "") return;
    onsave(trimmed.slice(0, 120));
    open = false;
  }
</script>

<Dialog bind:open title="Rename schedule" size="sm">
  <form id="rename-schedule" onsubmit={save}>
    <TextInput
      bind:value
      label="Name"
      maxlength={120}
      error={trimmed === "" ? "Give the schedule a name." : undefined}
    />
  </form>
  {#snippet footer()}
    <Button onclick={() => (open = false)}>Cancel</Button>
    <Button variant="primary" type="submit" form="rename-schedule" disabled={trimmed === ""}>Save</Button>
  {/snippet}
</Dialog>
