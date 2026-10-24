<script lang="ts">
  import { page } from "$app/state";
  import InvitePage from "$lib/features/friends/InvitePage.svelte";
</script>

{#key page.params.token}
  <InvitePage token={page.params.token ?? ""} />
{/key}
