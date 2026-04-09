<script>
	import { Icon } from '@steeze-ui/svelte-icon';
	import { ChevronLeft, User, Calendar, Check } from '@steeze-ui/heroicons';
	import { BookmarkPlus, CalendarPlus, Users } from '@steeze-ui/lucide-icons';
</script>

<div class="justify-inital flex min-w-80 flex-col border-r-1 border-border p-6 text-fg-black">
	<div>
		<div class="flex flex-row items-center justify-between">
			<div class="text-lg">Profile</div>
			<button class="flex cursor-pointer flex-row justify-end">
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
		<div class="flex flex-row items-center gap-2 py-3">
			<Icon src={BookmarkPlus} class="h-5 w-5 text-gray-500" />
			<span>Saved Courses</span>
		</div>
		<div class="flex flex-row items-center gap-2 py-3">
			<Icon src={Calendar} class="h-5 w-5 text-gray-500" />
			<span>Currently Taking</span>
		</div>
		<div class="flex flex-row items-center gap-2 py-3">
			<Icon src={Check} class="h-5 w-5 text-gray-500" />
			<span>Courses Taken</span>
		</div>
		<div class="flex flex-row items-center gap-2 py-3">
			<Icon src={CalendarPlus} class="h-5 w-5 text-gray-500" />
			<span>Schedule</span>
		</div>
		<div class="flex flex-row items-center gap-2 py-3">
			<Icon src={Users} class="h-5 w-5 text-gray-500" />
			<span>Friends</span>
		</div>
	</div>
</div>
