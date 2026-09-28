<script lang="ts">
	import { onMount } from 'svelte';
	import { getFrame, send } from '$lib/api';
	import { applyEvents, emptyDecorations } from '$lib/decorations';
	import type { Command, Frame, NodeId, UiAction } from '$lib/types';
	import Controls from '$lib/components/Controls.svelte';
	import GraphView from '$lib/components/GraphView.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';

	let frame: Frame | null = $state(null);
	let loading = $state(false);
	let error: string | null = $state(null);
	let tickIntervalMs = $state(200);

	let decorations = $derived.by(() => (frame ? applyEvents(frame.events) : emptyDecorations()));
	let showSidebar = $derived.by(
		() => frame !== null && (frame.selected !== null || decorations.sidebar !== null)
	);

	onMount(async () => {
		try {
			frame = await getFrame();
		} catch (e) {
			error = `Failed to connect to server: ${e}`;
		}
	});

	async function dispatch(command: Command) {
		loading = true;
		try {
			frame = await send(command);
			error = null;
		} catch (e) {
			error = `${e}`;
		}
		loading = false;
	}

	const ui = (action: UiAction) => dispatch({ type: 'Ui', action });
	const select = (node: NodeId | null) => ui({ type: 'SelectNode', node });
</script>

<svelte:head>
	<title>simviz</title>
</svelte:head>

{#if error}
	<div class="error">{error}</div>
{/if}

{#if frame}
	<div class="layout">
		<div class="left-panel">
			<Controls
				step={frame.step}
				{loading}
				onforward={() => dispatch({ type: 'StepForward' })}
				onback={() => dispatch({ type: 'StepBack' })}
				onreset={() => dispatch({ type: 'Reset' })}
				onspeedchange={(ms) => (tickIntervalMs = ms)}
			/>
		</div>
		<div class="graph-area">
			<GraphView topology={frame.topology} {decorations} selected={frame.selected} {tickIntervalMs} onselect={select} />
		</div>
		{#if showSidebar}
			<div class="right-panel">
				<Sidebar selected={frame.selected} content={decorations.sidebar} onaction={ui} onclose={() => select(null)} />
			</div>
		{/if}
	</div>
{:else if !error}
	<div class="loading">Loading...</div>
{/if}

<style>
	:global(html),
	:global(body) {
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
		width: 220px;
		min-width: 220px;
		background: #161b22;
		border-right: 1px solid #333;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
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
