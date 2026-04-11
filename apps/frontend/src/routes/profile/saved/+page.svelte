<script lang="ts">
	import Header from '@/lib/components/Header.svelte';
	import Footer from '@/lib/components/Footer.svelte';
	import ProfileSidebar from '@/lib/components/ProfileSidebar.svelte';
</script>

<Header />
<main class="flex flex-1 flex-col">
	<div class="flex flex-1 flex-row">
		<ProfileSidebar />
		<div class="flex flex-col">
			<div>
				<div>Saved Courses</div>
				<div>
					<div>Deparment</div>
					<div>Course Level</div>
				</div>
			</div>
			<div>bottom</div>
		</div>
	</div>
</main>
<Footer />
