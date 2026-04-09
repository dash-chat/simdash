<script lang="ts">
	import { onMount } from 'svelte';
	import { getState, stepForward, stepBack, reset, setBehavior } from '$lib/api';
	import { computeMetrics } from '$lib/metrics';
	import type { SimSnapshot, NodeInfo, OpTraceData } from '$lib/types';
	import Controls from '$lib/components/Controls.svelte';
	import GraphView from '$lib/components/GraphView.svelte';
	import MetricsPanel from '$lib/components/MetricsPanel.svelte';
	import NodeDetail from '$lib/components/NodeDetail.svelte';

	let snapshot: SimSnapshot | null = $state(null);
	let prevSnapshot: SimSnapshot | null = $state(null);
	let loading = $state(false);
	let selectedNodeId: number | null = $state(null);
	let selectedGroupId: number | null = $state(null);
	let selectedOpId: number | null = $state(null);
	let error: string | null = $state(null);

	let metrics = $derived(snapshot ? computeMetrics(snapshot) : null);
	let selectedNode: NodeInfo | null = $derived.by(() => {
		if (!snapshot) return null;
		return snapshot.nodes.find((n) => n.id === selectedNodeId) ?? null;
	});
	let highlightedNodeIds: Set<number> = $derived.by(() => {
		if (selectedGroupId === null || !snapshot) return new Set();
		return new Set(
			snapshot.nodes.filter((n) => n.groups.includes(selectedGroupId!)).map((n) => n.id)
		);
	});
	let opTrace: OpTraceData | null = $derived.by(() => {
		if (selectedOpId === null || !snapshot) return null;
		const holders = new Set(
			snapshot.nodes.filter((n) => n.ops.some((op) => op.id === selectedOpId)).map((n) => n.id)
		);
		const prevHolderSet = prevSnapshot
			? new Set(prevSnapshot.nodes.filter((n) => n.ops.some((op) => op.id === selectedOpId)).map((n) => n.id))
			: holders;
		const newHolders = new Set([...holders].filter((id) => !prevHolderSet.has(id)));
		return { holders, newHolders, prevHolders: prevHolderSet };
	});

	onMount(async () => {
		try {
			snapshot = await getState();
		} catch (e) {
			error = `Failed to connect to server: ${e}`;
		}
	});

	async function handleForward() {
		loading = true;
		try {
			prevSnapshot = snapshot;
			snapshot = await stepForward();
		} catch (e) {
			error = `${e}`;
		}
		loading = false;
	}

	async function handleBack() {
		loading = true;
		try {
			prevSnapshot = snapshot;
			snapshot = await stepBack();
		} catch (e) {
			error = `${e}`;
		}
		loading = false;
	}

	async function handleReset() {
		loading = true;
		try {
			prevSnapshot = null;
			snapshot = await reset();
			selectedNodeId = null;
		} catch (e) {
			error = `${e}`;
		}
		loading = false;
	}

	async function handleBehaviorChange(behavior: string) {
		loading = true;
		try {
			prevSnapshot = null;
			snapshot = await setBehavior(behavior);
			selectedNodeId = null;
			selectedOpId = null;
			selectedGroupId = null;
		} catch (e) {
			error = `${e}`;
		}
		loading = false;
	}

	function toggleOp(id: number | null) {
		selectedOpId = selectedOpId === id ? null : id;
	}
</script>

<svelte:head>
	<title>simdash viewer</title>
</svelte:head>

{#if error}
	<div class="error">{error}</div>
{/if}

{#if snapshot && metrics}
	<div class="layout">
		<div class="left-panel">
			<div class="metrics-area">
				<MetricsPanel {metrics} {selectedGroupId} {selectedOpId} onselectgroup={(id) => (selectedGroupId = id)} onselectop={toggleOp} />
			</div>
			<div class="controls-area">
				<Controls
					step={snapshot.step}
					{loading}
					behavior={snapshot.behavior}
					availableBehaviors={snapshot.available_behaviors}
					onforward={handleForward}
					onback={handleBack}
					onreset={handleReset}
					onbehaviorchange={handleBehaviorChange}
				/>
			</div>
		</div>
		<div class="graph-area">
			<GraphView {snapshot} {selectedNodeId} {highlightedNodeIds} {opTrace} onselect={(id) => { selectedNodeId = id; selectedGroupId = null; selectedOpId = null; }} />
		</div>
		{#if selectedNode}
			<div class="right-panel">
				<NodeDetail node={selectedNode} {selectedGroupId} {selectedOpId} onselectgroup={(id) => (selectedGroupId = id)} onselectop={toggleOp} onclose={() => (selectedNodeId = null)} />
			</div>
		{/if}
	</div>
{:else if !error}
	<div class="loading">Loading...</div>
{/if}

<style>
	:global(html), :global(body) {
		margin: 0;
		padding: 0;
		overflow: hidden;
		height: 100%;
		background: #0d1117;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
	}
	.error {
		padding: 12px;
		background: #5c1a1a;
		color: #ff8080;
		text-align: center;
	}
	.loading {
		color: #666;
		text-align: center;
		padding: 40px;
	}
	.layout {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}
	.left-panel {
		width: 280px;
		min-width: 280px;
		background: #161b22;
		border-right: 1px solid #333;
		display: flex;
		flex-direction: column;
	}
	.metrics-area {
		flex: 1;
		overflow-y: auto;
	}
	.controls-area {
		border-top: 1px solid #333;
	}
	.graph-area {
		flex: 1;
		overflow: hidden;
	}
	.right-panel {
		width: 320px;
		min-width: 320px;
		background: #161b22;
		border-left: 1px solid #333;
		overflow-y: auto;
	}
</style>
