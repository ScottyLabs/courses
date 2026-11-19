<script lang="ts">
	import { Hash, Clock, SquareArrowOutUpRight } from '@lucide/svelte';

	let { code, name, department, units, hoursPerWeek }: {
		code: string;
		name: string;
		department: string;
		units: number;
		hoursPerWeek?: number;
	} = $props();
</script>

<button class="min-w-0 cursor-pointer rounded-[10px] border border-border bg-bg-primary text-left md:rounded-md">
	<div class="flex flex-col justify-between gap-2 p-5">
		<div class="grid grid-cols-[max-content_1fr] items-center gap-x-2 gap-y-1 md:grid-cols-[max-content_1fr_auto] md:items-start">
			<span class="font-bold text-fg-black">{code}</span>
			<SquareArrowOutUpRight class="size-5 text-fg-black md:col-start-3 md:row-start-1" />
			<span class="col-span-2 min-w-0 font-medium wrap-break-word text-fg-black md:col-span-1 md:col-start-2 md:row-start-1">
				{name}
			</span>
		</div>
		<span class="text-sm font-semibold text-fg-gray">{department}</span>
		<div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm md:text-base">
			<div class="flex items-center gap-1">
				<Hash class="size-5 shrink-0" />
				<span class="font-semibold text-fg-black">{units} units</span>
			</div>
			{#if hoursPerWeek !== undefined}
				<div class="h-5 border-l border-border"></div>
				<div class="flex items-center gap-1">
					<Clock class="size-5 shrink-0" />
					<span class="font-semibold text-fg-black">{hoursPerWeek} hrs/week</span>
				</div>
			{/if}
		</div>
	</div>
</button>
