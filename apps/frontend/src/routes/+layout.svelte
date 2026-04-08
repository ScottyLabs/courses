<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Geist+Mono:wght@300..700&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

{@render children()}

<style>
	:global(:root) {
		--color-white: #ffffff;
		--color-gray: #f0f4f8;
	}

	:global(body) {
		margin: 0;
		padding: 0;

		font-family: 'Inter', sans-serif;
	}
</style>
