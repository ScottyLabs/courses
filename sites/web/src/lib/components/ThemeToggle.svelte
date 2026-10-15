<script>
	import { Sun, Moon } from '@lucide/svelte';

	let dark = $state(false);

	function toggle() {
		dark = !dark;
		document.documentElement.classList.toggle('dark', dark);
	}
</script>

<button
	onclick={toggle}
	class="relative flex h-7 w-14 cursor-pointer items-center rounded-full border-1 border-fg-gray transition-colors duration-300"
	aria-label="Toggle theme"
>
	<span
		class="absolute flex size-6 items-center justify-center rounded-full bg-fg-gray shadow transition-all duration-300 {dark
			? 'left-[calc(100%-1.75rem)]'
			: 'left-0.5'}"
	>
		{#if dark}
			<Moon fill="currentColor" class="size-4 text-white" />
		{:else}
			<Sun class="size-4 text-white" />
		{/if}
	</span>
</button>
