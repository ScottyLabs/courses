<script lang="ts">
	import { Hash, Clock, SquareArrowOutUpRight } from '@lucide/svelte';
</script>

<button class="h-29 cursor-pointer rounded-md border-1 border-border bg-bg-primary">
	<div class="flex flex-col justify-between p-5">
		<div class="flex flex-row justify-between">
			<div class="gap-2">
				<span class="font-bold text-fg-black">15-122</span>
				<span class="font-medium text-fg-black">Principles of Imperative Computation</span>
			</div>
			<SquareArrowOutUpRight class="size-5" />
		</div>
		<div class="flex flex-row items-center justify-between">
			<span class="text-sm font-semibold text-fg-gray">Computer Science</span>
			<div class="flex flex-row items-center gap-1">
				<div class="size-6 rounded-full bg-border text-white">C</div>
				<div class="size-6 rounded-full bg-border text-white">A</div>
				<span class="text-sm font-medium">+2</span>
			</div>
		</div>
		<div class="flex-start flex flex-row gap-3">
			<div class="flex flex-row items-center gap-1">
				<Hash class="size-5" />
				<span class="font-semibold text-fg-black">12 units</span>
			</div>
			<div class="rounded-md border-1 border-border"></div>
			<div class="flex flex-row items-center gap-1">
				<Clock class="size-5" />
				<span class="font-semibold text-fg-black">15.5 hrs/week</span>
			</div>
		</div>
	</div>
</button>
