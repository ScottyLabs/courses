<script lang="ts">
  import { Link, Plus } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import AddFriendDialog from "./AddFriendDialog.svelte";
  import { friendDialogs } from "./dialogs.svelte";
  import InviteDialog from "./InviteDialog.svelte";

  // The Friends heading's buttons (Figma "Share Link", "Add Friends") and their dialogs.
</script>

<Button size="sm" icon={Link} class="max-sm:h-10" onclick={() => (friendDialogs.inviting = true)}>Share link</Button>
<Button size="sm" icon={Plus} class="max-sm:h-10" onclick={() => (friendDialogs.adding = true)}>Add friends</Button>

<AddFriendDialog
  bind:open={friendDialogs.adding}
  oninvite={() => {
    friendDialogs.adding = false;
    friendDialogs.inviting = true;
  }}
/>
<InviteDialog bind:open={friendDialogs.inviting} />
