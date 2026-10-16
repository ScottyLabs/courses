<script>
	import { MessageSquareText } from '@lucide/svelte';
</script>

<footer
	class="flex min-h-16 flex-row items-center justify-between border-t-1 border-border bg-bg-secondary px-8 py-4"
>
	<div class="text-normal flex flex-col text-sm text-fg-gray">
		<p>Designed, developed and maintained with ❤️ by ScottyLabs.</p>
		<p>
			Want to help us make CMU Courses better? Join us at ScottyLabs or make a pull request on our
			Github repository.
		</p>
	</div>
	<button
		class="flex cursor-pointer items-center gap-2 rounded-full border border-fg-light-gray px-3 py-2 text-fg-black transition-colors duration-200 hover:border-fg-black"
	>
		<MessageSquareText class="size-5" />
		<span class="font-medium">Feedback</span>
	</button>
</footer>
