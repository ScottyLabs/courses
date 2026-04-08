<script>
	import { Icon } from '@steeze-ui/svelte-icon';
	import { MagnifyingGlass, UserCircle, Clock } from '@steeze-ui/heroicons';
	import favicon from '@/lib/assets/favicon.ico';
</script>

<header class="flex h-16 flex-row items-center justify-between bg-bg-secondary p-6 shadow-md">
	<a href="/" class="flex flex-initial flex-row justify-start hover:underline">
		<div class="flex items-center">
			<img src={favicon} class="h-8 w-8 rounded" alt="favicon" />
			<span class="ml-2 font-medium text-fg-black">CMU Courses</span>
		</div>
	</a>

	<div class="flex flex-row items-center justify-between gap-6">
		<a href="/courses" class="flex flex-row items-center">
			<Icon src={MagnifyingGlass} class="h-5 w-5 [stroke-width:2.5] text-fg-gray" />
			<span class="ml-2 font-medium text-fg-gray">Courses</span>
		</a>
		<a href="/courses" class="flex flex-row items-center">
			<Icon src={UserCircle} class="h-5 w-5 [stroke-width:2.5] text-fg-gray" />
			<span class="ml-2 font-medium text-fg-gray">Instructors</span>
		</a>
		<a href="/courses" class="flex flex-row items-center">
			<Icon src={Clock} class="h-5 w-5 [stroke-width:2.5] text-fg-gray" />
			<span class="ml-2 font-medium text-fg-gray">Schedules</span>
		</a>
		<a href="/courses" class="flex flex-row items-center">
			<Icon src={UserCircle} class="h-5 w-5 [stroke-width:2.5] text-fg-gray" />
			<span class="ml-2 font-medium text-fg-gray">Profile</span>
		</a>
	</div>

	<div>
		<p>log in</p>
	</div>
</header>
