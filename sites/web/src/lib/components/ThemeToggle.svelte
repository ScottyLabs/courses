<script>
	import { Icon } from '@steeze-ui/svelte-icon';
	import { Sun, Moon } from '@steeze-ui/heroicons';

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
		class="absolute flex h-6 w-6 items-center justify-center rounded-full bg-fg-gray shadow transition-all duration-300 {dark
			? 'left-[calc(100%-1.75rem)]'
			: 'left-0.5'}"
	>
		{#if dark}
			<Icon src={Moon} theme="solid" class="h-4 w-4 text-white" />
		{:else}
			<Icon src={Sun} class="h-4 w-4 text-white" />
		{/if}
	</span>
</button>
