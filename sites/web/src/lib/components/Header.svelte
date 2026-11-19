<script lang="ts">
	import { page } from '$app/state';
	import { Search, CircleUser, Clock, User, ChevronRight } from '@lucide/svelte';
	import favicon from '@/lib/assets/favicon.ico';
	import signOutIcon from '$lib/assets/sign-out.svg';
	import ThemeToggle from '@/lib/components/ThemeToggle.svelte';

	import { refreshSession, session } from '$lib/session';

	const pathname = $derived(page.url.pathname);
	const initials = $derived(
		$session.status === 'signed-in'
			? $session.name.trim().split(/\s+/, 2).map((part) => part.charAt(0)).join('').toUpperCase()
			: ''
	);
	let navigation = $state<HTMLElement>();
	let canScrollRight = $state(false);

	function updateScrollIndicator() {
		const lastTab = navigation?.lastElementChild;
		if (!navigation || !lastTab) return;
		canScrollRight =
			lastTab.getBoundingClientRect().right > navigation.getBoundingClientRect().right + 1;
	}

	function scrollRight() {
		if (!navigation) return;
		const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		navigation.scrollBy({
			left: navigation.clientWidth / 2,
			behavior: reduceMotion ? 'instant' : 'smooth'
		});
	}

	$effect(() => {
		if (!navigation) return;
		const observer = new ResizeObserver(updateScrollIndicator);
		observer.observe(navigation);
		return () => observer.disconnect();
	});

	$effect(() => {
		pathname;
		const activeTab = navigation?.querySelector<HTMLElement>('[aria-current]');
		if (navigation && activeTab) {
			navigation.scrollLeft += activeTab.getBoundingClientRect().right -
				navigation.getBoundingClientRect().right + 20;
		}
		updateScrollIndicator();
	});
</script>

{#snippet navLink(href: string, label: string, Icon: typeof Search)}
	{@const active = pathname === href || (href !== '/' && pathname.startsWith(href + '/'))}
	<a
		{href}
		aria-current={active ? (pathname === href ? 'page' : 'location') : undefined}
		class="flex min-h-11 shrink-0 items-center gap-2 whitespace-nowrap text-fg-gray aria-[current]:text-sky-700 lg:aria-[current]:text-fg-gray"
	>
		<Icon class="size-6" />
		<span class="font-medium">{label}</span>
	</a>
{/snippet}

<header class="grid min-h-16 grid-cols-[1fr_auto_1fr] items-center gap-y-1 bg-bg-primary pt-3 shadow-md lg:flex lg:justify-between lg:gap-4 lg:bg-bg-secondary lg:p-6">
	<a href="/" aria-label="CMU Courses home" class="col-start-2 row-start-1 flex justify-center hover:underline lg:justify-start">
		<div class="flex items-center gap-3">
			<img src={favicon} class="size-10 rounded-xl lg:size-8 lg:rounded" alt="" />
			<span class="hidden font-medium whitespace-nowrap text-fg-black lg:inline">CMU Courses</span>
		</div>
	</a>

	<div class="relative col-span-3 row-start-2 min-w-0">
		<nav
			id="main-navigation"
			bind:this={navigation}
			onscroll={updateScrollIndicator}
			aria-label="Main navigation"
			class="flex min-w-0 items-center gap-8 overflow-x-auto px-5 pt-2 [scrollbar-width:none] lg:gap-5 lg:p-0 xl:gap-8"
		>
			{@render navLink('/courses', 'Courses', Search)}
			{@render navLink('/instructors', 'Instructors', CircleUser)}
			{@render navLink('/schedules', 'Schedules', Clock)}
			{@render navLink('/profile', 'Profile', User)}
		</nav>
		{#if canScrollRight}
			<button
				type="button"
				onclick={scrollRight}
				aria-label="Scroll navigation right"
				aria-controls="main-navigation"
				class="absolute top-2 right-0 bottom-0 grid w-9 cursor-pointer place-items-center bg-linear-to-r from-transparent via-bg-primary to-bg-primary text-fg-black lg:top-0 lg:via-bg-secondary lg:to-bg-secondary"
			>
				<ChevronRight class="size-4" />
			</button>
		{/if}
	</div>

	<div class="contents lg:flex lg:items-center lg:gap-4 xl:gap-8">
		<div class="col-start-1 row-start-1 ml-5 justify-self-start lg:m-0">
			<ThemeToggle />
		</div>
		<div class="col-start-3 row-start-1 mr-5 min-w-0 justify-self-end lg:m-0">
			{#if $session.status === 'signed-in'}
				<div class="flex items-center gap-1 lg:gap-3">
					<span class="grid size-8 shrink-0 place-items-center rounded-full bg-border text-xs text-fg-black lg:hidden" title={$session.name}>
						{initials}
					</span>
					<span class="hidden max-w-28 truncate text-sm font-medium text-fg-black xl:inline" title={$session.name}>{$session.name}</span>
					<a href="/auth/logout" aria-label="Sign out" data-sveltekit-reload
						class="flex size-9 items-center justify-center rounded-full text-sm font-semibold whitespace-nowrap text-fg-black hover:border-fg-black lg:h-auto lg:w-auto lg:gap-2 lg:border lg:border-fg-light-gray lg:px-3 lg:py-2">
						<img src={signOutIcon} alt="" width="20" height="20" />
						<span class="hidden lg:inline">Sign out</span>
					</a>
				</div>
			{:else if $session.status === 'signed-out'}
				<a href="/auth/login" data-sveltekit-reload
					class="inline-flex min-h-9 items-center rounded-full border border-fg-light-gray px-2 py-1 text-center text-xs font-semibold text-fg-black hover:border-fg-black lg:px-4 lg:py-2 lg:text-sm">
					Sign in with CMU
				</a>
			{:else if $session.status === 'unavailable'}
				<button type="button" onclick={refreshSession} class="min-h-9 text-xs text-fg-black underline lg:text-sm">
					Retry sign-in status
				</button>
			{/if}
		</div>
	</div>
</header>
