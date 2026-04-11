<script lang="ts">
	import Header from '@/lib/components/Header.svelte';
	import Footer from '@/lib/components/Footer.svelte';
	import ProfileSidebar from '@/lib/components/ProfileSidebar.svelte';
	import { Icon } from '@steeze-ui/svelte-icon';
	import { ChevronUp } from '@steeze-ui/heroicons';
</script>

<Header />
<main class="flex flex-1 flex-col">
	<div class="flex flex-1 flex-row">
		<ProfileSidebar />
		<div class="flex min-w-full flex-col">
			<div class="flex flex-col gap-6 border-b-1 border-border p-8">
				<div class="text-xl text-fg-black">Saved Courses</div>
				<div class="flex flex-row gap-3">
					<button
						class="flex cursor-pointer flex-row gap-12 rounded-sm border-1 border-border px-3 py-2 text-fg-gray transition-colors duration-200 hover:border-fg-black"
					>
						<div class="text-sm">Deparment</div>
						<Icon src={ChevronUp} class="h-5 w-5 [stroke-width:2]" />
					</button>
					<button
						class="flex cursor-pointer flex-row gap-12 rounded-sm border-1 border-border px-3 py-2 text-fg-gray transition-colors duration-200 hover:border-fg-black"
					>
						<div class="text-sm">Course Level</div>
						<Icon src={ChevronUp} class="h-5 w-5 [stroke-width:2]" />
					</button>
				</div>
			</div>
			<div class="grid flex-1 grid-cols-2 bg-bg-secondary">bottom</div>
		</div>
	</div>
</main>
<Footer />
