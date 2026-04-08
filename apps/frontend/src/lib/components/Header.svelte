<script>
	import { Icon } from '@steeze-ui/svelte-icon';
	import { MagnifyingGlass } from '@steeze-ui/heroicons';
	import favicon from '@/lib/assets/favicon.ico';
</script>

<header class="flex h-16 flex-row items-center justify-between bg-gray p-6 shadow-md">
	<a href="/" class="text-dark-gray-500 flex flex-initial flex-row justify-start hover:underline">
		<div class="flex items-center">
			<img src={favicon} class="h-8 w-8 rounded" alt="favicon" />
			<h6 class="ml-2 font-medium text-dark-gray">CMU Courses</h6>
		</div>
	</a>

	<div>
		<a href="/courses" class="flex flex-row items-center">
			<Icon src={MagnifyingGlass} class="h-5 w-5" />
			<h6 class="ml-2 font-medium text-dark-gray">Courses</h6>
		</a>
	</div>

	<div>
		<p>log in</p>
	</div>
</header>
