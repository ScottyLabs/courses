<script lang="ts">
	import { BookmarkPlus, ChevronUp } from '@lucide/svelte';
</script>

<div class="flex min-w-0 flex-1 flex-col">
	<div class="flex flex-col gap-4 bg-bg-secondary px-5 pt-4 md:gap-6 md:border-b md:border-border md:bg-bg-primary md:p-8">
		<h1 class="flex items-center gap-3 text-fg-black md:text-xl">
			<BookmarkPlus class="size-5 md:hidden" />
			Saved Courses
		</h1>
		<div class="flex flex-wrap gap-3">
			<button
				class="flex cursor-pointer items-center gap-4 md:gap-8 xl:gap-12 rounded-sm border-1 border-border px-3 py-2 text-fg-gray transition-colors duration-200 hover:border-fg-black"
			>
				<div class="text-sm">Department</div>
				<ChevronUp class="size-5" />
			</button>
			<button
				class="flex cursor-pointer items-center gap-4 md:gap-8 xl:gap-12 rounded-sm border-1 border-border px-3 py-2 text-fg-gray transition-colors duration-200 hover:border-fg-black"
			>
				<div class="text-sm">Course Level</div>
				<ChevronUp class="size-5" />
			</button>
		</div>
	</div>
	<div class="flex-1 bg-bg-secondary"></div>
</div>
