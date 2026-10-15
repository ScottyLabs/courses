<script>
	import {
		ChevronLeft,
		ChevronRight,
		User,
		Calendar,
		Check,
		BookmarkPlus,
		CalendarPlus,
		Users
	} from '@lucide/svelte';

	let hidden = $state(false);
	function toggleHidden() {
		hidden = !hidden;
	}
</script>

{#if hidden}
	<div class="justify-initial flex min-w-16 flex-col border-r-1 border-border py-6 text-fg-black">
		<button onclick={toggleHidden} class="cursor-pointer px-6 pb-34">
			<ChevronRight class="size-5" />
		</button>
		<a
			href="profile/saved"
			class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<BookmarkPlus class="size-5 text-gray-500" />
		</a>
		<a
			href="profile/current"
			class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<Calendar class="size-5 text-gray-500" />
		</a>
		<a
			href="profile/completed"
			class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<Check class="size-5 text-gray-500" />
		</a>
		<a
			href="profile/schedule"
			class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<CalendarPlus class="size-5 text-gray-500" />
		</a>
		<a
			href="profile/friends"
			class="cursor-pointer px-6 py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<Users class="size-5 text-gray-500" />
		</a>
	</div>
{:else}
	<div class="justify-inital flex min-w-80 flex-col border-r-1 border-border py-6 text-fg-black">
		<div class="px-6">
			<div class="flex flex-row items-center justify-between">
				<div class="text-lg">Profile</div>
				<button onclick={toggleHidden} class="flex cursor-pointer flex-row justify-end">
					<span class="text-semibold text-sm">Hide</span>
					<ChevronLeft class="size-5" />
				</button>
			</div>
			<div class="my-6 flex flex-row items-center gap-4">
				<div class="flex size-20 items-center justify-center rounded-lg bg-gray-200">
					<User class="size-8 text-gray-500" />
				</div>
				<div class="flex flex-col">
					<span class="text-lg">Andrew Carnegie</span>
					<span class="text-sm">Computer Science</span>
				</div>
			</div>
		</div>
		<div>
			<a
				href="profile/saved"
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<BookmarkPlus class="size-5 text-gray-500" />
				<span>Saved Courses</span>
			</a>
			<a
				href="profile/current"
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Calendar class="size-5 text-gray-500" />
				<span>Currently Taking</span>
			</a>
			<a
				href="profile/completed"
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Check class="size-5 text-gray-500" />
				<span>Courses Taken</span>
			</a>
			<a
				href="profile/schedule"
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<CalendarPlus class="size-5 text-gray-500" />
				<span>Schedule</span>
			</a>
			<a
				href="profile/friends"
				class="flex w-full cursor-pointer flex-row items-center gap-2 px-6 py-3 transition-colors duration-200 hover:bg-border"
			>
				<Users class="size-5 text-gray-500" />
				<span>Friends</span>
			</a>
		</div>
	</div>
{/if}
