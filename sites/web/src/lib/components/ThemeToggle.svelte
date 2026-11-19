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
	class="relative flex h-6 w-[42px] lg:h-7 lg:w-14 cursor-pointer items-center rounded-full border-1 border-fg-gray transition-colors duration-300"
	aria-label="Toggle theme"
	aria-pressed={dark}
>
	<span
		class="absolute flex size-[18px] items-center lg:size-6 justify-center rounded-full bg-fg-gray shadow transition-all duration-300 {dark
			? 'left-[calc(100%-22px)] lg:left-[calc(100%-1.75rem)]'
			: 'left-0.5'}"
	>
		{#if dark}
			<Moon fill="currentColor" class="size-3 text-white lg:size-4" />
		{:else}
			<Sun class="size-3 text-white lg:size-4" />
		{/if}
	</span>
</button>
