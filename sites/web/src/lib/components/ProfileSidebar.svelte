<script>
	import { page } from '$app/state';
	import { session } from '$lib/session';
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
	const showingSaved = $derived(page.route.id === "/profile/saved");
	function toggleHidden() {
		hidden = !hidden;
	}
</script>

{#if hidden}
	<div class="justify-initial flex flex-wrap items-center border-b border-border md:min-w-16 md:flex-col md:border-r md:border-b-0 md:py-6 text-fg-black" class:profile-compact={showingSaved}>
		<button aria-expanded={!hidden} aria-label={hidden ? "Show profile menu" : "Hide profile menu"} onclick={toggleHidden} class="cursor-pointer px-3 py-3 md:px-6 md:py-0 md:pb-34">
			<ChevronRight class="size-5" />
		</button>
		<a
			href="/profile/saved"
			aria-label="Saved Courses"
			class="cursor-pointer px-3 py-3 md:px-6 md:py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<BookmarkPlus class="size-5 text-gray-500" />
		</a>
		<a
			href="/profile/current"
			aria-label="Currently Taking"
			class="cursor-pointer px-3 py-3 md:px-6 md:py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<Calendar class="size-5 text-gray-500" />
		</a>
		<a
			href="/profile/completed"
			aria-label="Courses Taken"
			class="cursor-pointer px-3 py-3 md:px-6 md:py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<Check class="size-5 text-gray-500" />
		</a>
		<a
			href="/profile/schedule"
			aria-label="Schedule"
			class="cursor-pointer px-3 py-3 md:px-6 md:py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<CalendarPlus class="size-5 text-gray-500" />
		</a>
		<a
			href="/profile/friends"
			aria-label="Friends"
			class="cursor-pointer px-3 py-3 md:px-6 md:py-3.5 transition-colors duration-200 hover:bg-border"
		>
			<Users class="size-5 text-gray-500" />
		</a>
	</div>
{:else}
	<div class="justify-inital flex min-w-0 flex-col border-b border-border py-4 md:min-w-64 md:border-r md:border-b-0 md:py-6 lg:min-w-80 text-fg-black" class:profile-below={showingSaved}>
		<div class="px-5 md:px-6" class:profile-summary={showingSaved}>
			<div class="flex flex-row items-center justify-between">
				<div class="text-lg">Profile</div>
				<button aria-expanded={!hidden} aria-label={hidden ? "Show profile menu" : "Hide profile menu"} onclick={toggleHidden} class="flex cursor-pointer flex-row justify-end">
					<span class="text-semibold text-sm">Hide</span>
					<ChevronLeft class="size-5" />
				</button>
			</div>
			{#if $session.status === 'signed-in'}
				<div class="my-4 flex items-center gap-4 md:my-6">
					<div class="flex size-12 shrink-0 items-center justify-center rounded-lg bg-gray-200 md:size-20">
						<User class="size-8 text-gray-500" />
					</div>
					<span class="min-w-0 wrap-break-word text-lg">{$session.name}</span>
				</div>
			{/if}
		</div>
		<div>
			<a
				href="/profile/saved"
				class:profile-summary={showingSaved}
				class="flex w-full cursor-pointer flex-row items-center gap-3 px-5 py-3 md:gap-2 md:px-6 transition-colors duration-200 hover:bg-border"
			>
				<BookmarkPlus class="size-5 text-gray-500" />
				<span>Saved Courses</span>
			</a>
			<a
				href="/profile/current"
				class="flex w-full cursor-pointer flex-row items-center gap-3 px-5 py-3 md:gap-2 md:px-6 transition-colors duration-200 hover:bg-border"
			>
				<Calendar class="size-5 text-gray-500" />
				<span>Currently Taking</span>
			</a>
			<a
				href="/profile/completed"
				class="flex w-full cursor-pointer flex-row items-center gap-3 px-5 py-3 md:gap-2 md:px-6 transition-colors duration-200 hover:bg-border"
			>
				<Check class="size-5 text-gray-500" />
				<span>Courses Taken</span>
			</a>
			<a
				href="/profile/schedule"
				class="flex w-full cursor-pointer flex-row items-center gap-3 px-5 py-3 md:gap-2 md:px-6 transition-colors duration-200 hover:bg-border"
			>
				<CalendarPlus class="size-5 text-gray-500" />
				<span>Schedule</span>
			</a>
			<a
				href="/profile/friends"
				class="flex w-full cursor-pointer flex-row items-center gap-3 px-5 py-3 md:gap-2 md:px-6 transition-colors duration-200 hover:bg-border"
			>
				<Users class="size-5 text-gray-500" />
				<span>Friends</span>
			</a>
		</div>
	</div>
{/if}

<style>
	@media (width < 48rem) {
		.profile-below,
		.profile-compact {
			order: 1;
			border-top: 1px solid var(--color-border);
			padding-block: 0;
		}

		.profile-below a {
			min-height: 3.75rem;
		}

		.profile-below a + a {
			border-top: 1px solid var(--color-border);
		}

		.profile-below .profile-summary {
			display: none;
		}
	}
</style>
