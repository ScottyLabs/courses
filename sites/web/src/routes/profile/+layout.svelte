<script lang="ts">
	import ProfileSidebar from '$lib/components/ProfileSidebar.svelte';

	let { children } = $props();
</script>

<main class="flex flex-1 flex-col">
	<div class="flex min-w-0 flex-1 flex-col md:flex-row">
		<ProfileSidebar />
		{@render children()}
	</div>
</main>
