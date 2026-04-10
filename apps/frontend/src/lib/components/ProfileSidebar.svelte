<script>
	import { Icon } from '@steeze-ui/svelte-icon';
	import { ChevronLeft, ChevronRight, User, Calendar, Check } from '@steeze-ui/heroicons';
	import { BookmarkPlus, CalendarPlus, Users } from '@steeze-ui/lucide-icons';

	let hidden = $state(false);
	function toggleHidden() {
		hidden = !hidden;
	}
</script>

{#if hidden}
	<div class="justify-initial flex min-w-16 flex-col border-r-1 border-border py-6 text-fg-black">
		<button onclick={toggleHidden} class="cursor-pointer px-6 pb-34">
			<Icon src={ChevronRight} class="h-5 w-5 [stroke-width:2]" />
		</button>
		<button class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border">
			<Icon src={BookmarkPlus} class="h-5 w-5 text-gray-500" />
		</button>
		<button class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border">
			<Icon src={Calendar} class="h-5 w-5 text-gray-500" />
		</button>
		<button class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border">
			<Icon src={Check} class="h-5 w-5 text-gray-500" />
		</button>
		<button class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border">
			<Icon src={CalendarPlus} class="h-5 w-5 text-gray-500" />
		</button>
		<button class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border">
			<Icon src={Users} class="h-5 w-5 text-gray-500" />
		</button>
	</div>
{:else}
	<div class="justify-inital flex min-w-80 flex-col border-r-1 border-border py-6 text-fg-black">
		<div class="px-6">
			<div class="flex flex-row items-center justify-between">
				<div class="text-lg">Profile</div>
				<button onclick={toggleHidden} class="flex cursor-pointer flex-row justify-end">
					<span class="text-semibold text-sm">Hide</span>
					<Icon src={ChevronLeft} class="h-5 w-5 [stroke-width:2]" />
				</button>
			</div>
			<div class="my-6 flex flex-row items-center gap-4">
				<div class="flex h-20 w-20 items-center justify-center rounded-lg bg-gray-200">
					<Icon src={User} class="h-8 w-8 text-gray-500" />
				</div>
				<div class="flex flex-col">
					<span class="text-lg">Andrew Carnegie</span>
					<span class="text-sm">Computer Science</span>
				</div>
			</div>
		</div>
		<div>
			<button
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Icon src={BookmarkPlus} class="h-5 w-5 text-gray-500" />
				<span>Saved Courses</span>
			</button>
			<button
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Icon src={Calendar} class="h-5 w-5 text-gray-500" />
				<span>Currently Taking</span>
			</button>
			<button
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Icon src={Check} class="h-5 w-5 text-gray-500" />
				<span>Courses Taken</span>
			</button>
			<button
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Icon src={CalendarPlus} class="h-5 w-5 text-gray-500" />
				<span>Schedule</span>
			</button>
			<button
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Icon src={Users} class="h-5 w-5 text-gray-500" />
				<span>Friends</span>
			</button>
		</div>
	</div>
{/if}
