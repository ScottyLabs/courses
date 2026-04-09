<script>
	import { Icon } from '@steeze-ui/svelte-icon';
	import { MagnifyingGlass, UserCircle, Clock, User } from '@steeze-ui/heroicons';
	import { LogOut } from '@steeze-ui/lucide-icons';
	import favicon from '@/lib/assets/favicon.ico';
	import ThemeToggle from '@/lib/components/ThemeToggle.svelte';
</script>

<header class="flex min-h-16 flex-row items-center justify-between bg-bg-secondary p-6 shadow-md">
	<a href="/" class="flex flex-initial flex-row justify-start hover:underline">
		<div class="flex items-center gap-3">
			<img src={favicon} class="h-8 w-8 rounded" alt="favicon" />
			<span class="font-medium text-fg-black">CMU Courses</span>
		</div>
	</a>

	<div class="flex flex-row items-center justify-between gap-8">
		<a href="/courses" class="flex flex-row items-center gap-2.5">
			<Icon src={MagnifyingGlass} class="h-6 w-6 [stroke-width:2] text-fg-gray" />
			<span class="font-medium text-fg-gray">Courses</span>
		</a>
		<a href="/instructors" class="flex flex-row items-center gap-2.5">
			<Icon src={UserCircle} class="h-6 w-6 [stroke-width:2] text-fg-gray" />
			<span class="font-medium text-fg-gray">Instructors</span>
		</a>
		<a href="/schedules" class="flex flex-row items-center gap-2.5">
			<Icon src={Clock} class="h-6 w-6 [stroke-width:2] text-fg-gray" />
			<span class="font-medium text-fg-gray">Schedules</span>
		</a>
		<a href="/profile" class="flex flex-row items-center gap-2.5">
			<Icon src={User} class="h-6 w-6 [stroke-width:2] text-fg-gray" />
			<span class="font-medium text-fg-gray">Profile</span>
		</a>
	</div>

	<div class="flex items-center gap-8">
		<ThemeToggle />
		<button
			class="flex cursor-pointer items-center gap-2 rounded-full border border-fg-light-gray px-3 py-2 text-fg-black transition-colors duration-200 hover:border-fg-black hover:text-fg-black"
		>
			<Icon src={LogOut} class="h-5 w-5 [stroke-width:2]" />
			<span class="font-medium">Sign out</span>
		</button>
	</div>
</header>
