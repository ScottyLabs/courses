<script>
	import { Search, CircleUser, Clock, User, LogOut } from '@lucide/svelte';
	import favicon from '@/lib/assets/favicon.ico';
	import ThemeToggle from '@/lib/components/ThemeToggle.svelte';
</script>

<header class="flex min-h-16 flex-row items-center justify-between bg-bg-secondary p-6 shadow-md">
	<a href="/" class="flex flex-initial flex-row justify-start hover:underline">
		<div class="flex items-center gap-3">
			<img src={favicon} class="size-8 rounded" alt="favicon" />
			<span class="font-medium text-fg-black">CMU Courses</span>
		</div>
	</a>

	<div class="flex flex-row items-center justify-between gap-8">
		<a href="/courses" class="flex flex-row items-center gap-2.5">
			<Search class="size-6 text-fg-gray" />
			<span class="font-medium text-fg-gray">Courses</span>
		</a>
		<a href="/instructors" class="flex flex-row items-center gap-2.5">
			<CircleUser class="size-6 text-fg-gray" />
			<span class="font-medium text-fg-gray">Instructors</span>
		</a>
		<a href="/schedules" class="flex flex-row items-center gap-2.5">
			<Clock class="size-6 text-fg-gray" />
			<span class="font-medium text-fg-gray">Schedules</span>
		</a>
		<a href="/profile" class="flex flex-row items-center gap-2.5">
			<User class="size-6 text-fg-gray" />
			<span class="font-medium text-fg-gray">Profile</span>
		</a>
	</div>

	<div class="flex items-center gap-8">
		<ThemeToggle />
		<button
			class="flex cursor-pointer items-center gap-2 rounded-full border border-fg-light-gray px-3 py-2 text-fg-black transition-colors duration-200 hover:border-fg-black hover:text-fg-black"
		>
			<LogOut class="size-5" />
			<span class="font-medium">Sign out</span>
		</button>
	</div>
</header>
