<script lang="ts">
	import { onMount } from 'svelte';
	import init, { WasmIndex } from '$lib/courses-index/courses_index_wasm.js';

	type Hit = { course_id: number; score: number };
	type QueryResult = {
		hits: Hit[];
		total_matched: number;
		did_you_mean_codes: string[];
	};
	type CourseSummary = { code: string; name: string; pagerank: number };

	let index: WasmIndex | null = $state(null);
	let loadMs = $state<number | null>(null);
	let fetchMs = $state<number | null>(null);
	let decompressMs = $state<number | null>(null);
	let buildMs = $state<number | null>(null);
	let courseCount = $state<number | null>(null);
	let query = $state('linear algebra');
	let lastResult = $state<QueryResult | null>(null);
	let lastQueryUs = $state<number | null>(null);
	let resolvedHits = $state<{ hit: Hit; course: CourseSummary }[]>([]);
	let bench = $state<{ label: string; hits: number; p50_us: number; p99_us: number }[]>([]);

	const catalogUrl = import.meta.env.VITE_CATALOG_URL ?? '/catalog.bin';

	onMount(async () => {
		await init();
		const t0 = performance.now();
		const response = await fetch(catalogUrl);
		const t1 = performance.now();
		const decompressed = await new Response(
			response.body!.pipeThrough(new DecompressionStream('gzip'))
		).arrayBuffer();
		const t2 = performance.now();
		const bytes = new Uint8Array(decompressed);
		index = new WasmIndex(bytes);
		const t3 = performance.now();

		fetchMs = t1 - t0;
		decompressMs = t2 - t1;
		buildMs = t3 - t2;
		loadMs = t3 - t0;
		courseCount = index.course_count();
		runQuery();
	});

	function runQuery() {
		if (!index) return;
		const t0 = performance.now();
		const res = index.query({ text: query, limit: 10 }) as QueryResult;
		lastQueryUs = (performance.now() - t0) * 1000;
		lastResult = res;
		resolvedHits = res.hits.map((hit) => {
			const c = index!.course_by_id(hit.course_id) as CourseSummary;
			return { hit, course: c };
		});
	}

	function runBench() {
		if (!index) return;
		const cases = [
			{ label: 'linear algebra', q: { text: 'linear algebra', limit: 10 } },
			{ label: 'machine learning', q: { text: 'machine learning', limit: 10 } },
			{ label: '15-122', q: { text: '15-122', limit: 10 } },
			{ label: 'imperative', q: { text: 'imperative', limit: 10 } },
			{ label: 'dept=15 (filter only)', q: { facets: { dept: ['15'] }, sort: 'PageRankDesc', limit: 10 } },
			{ label: 'algorithms + dept=15', q: { text: 'algorithms', facets: { dept: ['15'] }, limit: 10 } },
			{ label: 'browse top 50', q: { sort: 'PageRankDesc', limit: 50 } }
		];
		const out: typeof bench = [];
		for (const { label, q } of cases) {
			for (let i = 0; i < 50; i++) index.query(q);
			const samples: number[] = [];
			for (let i = 0; i < 1000; i++) {
				const t0 = performance.now();
				index.query(q);
				samples.push((performance.now() - t0) * 1000);
			}
			samples.sort((a, b) => a - b);
			const res = index.query(q) as QueryResult;
			out.push({
				label,
				hits: res.total_matched,
				p50_us: samples[Math.floor(samples.length / 2)],
				p99_us: samples[Math.floor(samples.length * 0.99)]
			});
		}
		bench = out;
	}
</script>

<main>
	<h1>courses-index wasm validation</h1>
	{#if loadMs === null}
		<p>Loading catalog...</p>
	{:else}
		<p>
			Loaded {courseCount} courses in {loadMs.toFixed(1)} ms (fetch {fetchMs?.toFixed(1)} + gunzip {decompressMs?.toFixed(1)}
			+ build {buildMs?.toFixed(1)}).
		</p>

		<section>
			<h2>Single query</h2>
			<input bind:value={query} oninput={runQuery} placeholder="search courses" />
			<p>
				Last query took {lastQueryUs?.toFixed(1)} µs and matched {lastResult?.total_matched} docs.
			</p>
			{#if lastResult?.did_you_mean_codes?.length}
				<p>did you mean: {lastResult.did_you_mean_codes.join(', ')}</p>
			{/if}
			<ol>
				{#each resolvedHits as row (row.hit.course_id)}
					<li>
						<strong>{row.hit.score.toFixed(3)}</strong>
						<code>{row.course?.code ?? '???'}</code>
						{row.course?.name ?? ''}
						<span class="pr">pr={row.course?.pagerank?.toFixed(5) ?? '?'}</span>
					</li>
				{/each}
			</ol>
		</section>

		<section>
			<h2>Benchmark</h2>
			<button onclick={runBench}>Run 1000-sample sweep</button>
			{#if bench.length}
				<table>
					<thead>
						<tr>
							<th>query</th>
							<th>hits</th>
							<th>p50 µs</th>
							<th>p99 µs</th>
						</tr>
					</thead>
					<tbody>
						{#each bench as row (row.label)}
							<tr>
								<td>{row.label}</td>
								<td>{row.hits}</td>
								<td>{row.p50_us.toFixed(1)}</td>
								<td>{row.p99_us.toFixed(1)}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</section>
	{/if}
</main>

<style>
	main {
		max-width: 48rem;
		margin: 2rem auto;
		font-family: ui-monospace, monospace;
		padding: 0 1rem;
	}
	input {
		font: inherit;
		padding: 0.4rem 0.6rem;
		width: 24rem;
		max-width: 100%;
	}
	table {
		border-collapse: collapse;
		margin-top: 0.5rem;
	}
	th,
	td {
		padding: 0.2rem 0.6rem;
		text-align: right;
		border-bottom: 1px solid #ccc;
	}
	th:first-child,
	td:first-child {
		text-align: left;
	}
	button {
		font: inherit;
		padding: 0.4rem 0.8rem;
		margin-bottom: 0.5rem;
	}
	.pr {
		color: #888;
		margin-left: 0.5rem;
	}
	code {
		background: #eee;
		padding: 0 0.3rem;
		border-radius: 2px;
	}
</style>
