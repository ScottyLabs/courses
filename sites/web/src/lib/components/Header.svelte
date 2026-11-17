<script lang="ts">
	import { Search, CircleUser, Clock, User } from '@lucide/svelte';
	import favicon from '@/lib/assets/favicon.ico';
	import signOutIcon from '$lib/assets/sign-out.svg';
	import ThemeToggle from '@/lib/components/ThemeToggle.svelte';

	import { refreshSession, session } from '$lib/session';
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
		{#if $session.status === 'signed-in'}
			<div class="flex items-center gap-3">
				<span class="text-sm font-medium text-fg-black">{$session.name}</span>
				<a href="/auth/logout" data-sveltekit-reload
					class="flex items-center gap-2 rounded-full border border-fg-light-gray px-4 py-[9px] text-sm font-semibold text-fg-black transition-colors hover:border-fg-black">
					<img src={signOutIcon} alt="" width="20" height="20" />
					<span>Sign out</span>
				</a>
			</div>
		{:else if $session.status === 'signed-out'}
			<a href="/auth/login" data-sveltekit-reload
				class="rounded-full border border-fg-light-gray px-4 py-[9px] text-sm font-semibold text-fg-black transition-colors hover:border-fg-black">
				Sign in with CMU
			</a>
		{:else if $session.status === 'unavailable'}
			<button type="button" onclick={refreshSession} class="text-sm text-fg-black underline">
				Retry sign-in status
			</button>
		{/if}
	</div>
</header>
