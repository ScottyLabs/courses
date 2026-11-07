<script lang="ts">
	import './layout.css';
	import Header from '$lib/components/Header.svelte';
	import Footer from '$lib/components/Footer.svelte';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<Header />
{@render children()}
<Footer />

<style>
	:global(body) {
		margin: 0;
		padding: 0;

		min-height: 100dvh;
		min-width: 100dvh;
		display: flex;
		flex-direction: column;

		font-family: 'Inter', sans-serif;
	}
</style>
