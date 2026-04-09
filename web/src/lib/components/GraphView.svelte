<script lang="ts">
	import { onMount } from 'svelte';
	import cytoscape from 'cytoscape';
	import type { SimSnapshot, OpTraceData } from '$lib/types';

	interface Props {
		snapshot: SimSnapshot;
		selectedNodeId: number | null;
		highlightedNodeIds: Set<number>;
		opTrace: OpTraceData | null;
		onselect: (nodeId: number | null) => void;
	}

	let { snapshot, selectedNodeId, highlightedNodeIds, opTrace, onselect }: Props = $props();

	let container: HTMLDivElement;
	let cy: cytoscape.Core | null = null;
	let layoutDone = false;

	// Count of unique ops across the whole network (for coloring)
	function uniqueOpCount(snapshot: SimSnapshot): number {
		const ids = new Set<number>();
		for (const node of snapshot.nodes) {
			for (const op of node.ops) {
				ids.add(op.id);
			}
		}
		return ids.size;
	}

	function snapshotToElements(snapshot: SimSnapshot): cytoscape.ElementDefinition[] {
		const nodes = snapshot.graph.nodes.map((n) => ({
			data: { id: `n${n.id}`, label: `${n.id}` }
		}));
		const edges = snapshot.graph.edges.map((e, i) => ({
			data: { id: `e${i}`, source: `n${e.source}`, target: `n${e.target}` }
		}));
		return [...nodes, ...edges];
	}

	function opSaturation(nodeOpCount: number, totalOps: number): string {
		if (totalOps === 0) return '#555';
		const ratio = nodeOpCount / totalOps;
		// Interpolate from red (0) to green (1)
		const r = Math.round(220 * (1 - ratio));
		const g = Math.round(220 * ratio);
		return `rgb(${r}, ${g}, 60)`;
	}

	function updateNodeStyles(snapshot: SimSnapshot) {
		if (!cy) return;
		const total = uniqueOpCount(snapshot);
		const nodeMap = new Map(snapshot.nodes.map((n) => [n.id, n]));
		const hasHighlight = highlightedNodeIds.size > 0;
		const tracing = opTrace !== null;

		cy.nodes().forEach((ele) => {
			const id = parseInt(ele.id().slice(1));
			const node = nodeMap.get(id);
			const opCount = node?.ops.length ?? 0;

			// Background color
			if (tracing) {
				if (opTrace!.newHolders.has(id)) {
					ele.style('background-color', '#40e0a0');
				} else if (opTrace!.holders.has(id)) {
					ele.style('background-color', '#4a90d9');
				} else {
					ele.style('background-color', '#555');
				}
			} else {
				ele.style('background-color', opSaturation(opCount, total));
			}

			// Border
			if (id === selectedNodeId) {
				ele.style('border-width', 3);
				ele.style('border-color', '#fff');
			} else if (hasHighlight && highlightedNodeIds.has(id)) {
				ele.style('border-width', 3);
				ele.style('border-color', '#f0c040');
			} else if (tracing && opTrace!.newHolders.has(id)) {
				ele.style('border-width', 3);
				ele.style('border-color', '#40e0a0');
			} else {
				ele.style('border-width', 0);
			}

			// Opacity
			if (tracing) {
				ele.style('opacity', opTrace!.holders.has(id) || id === selectedNodeId ? 1 : 0.2);
			} else if (hasHighlight) {
				ele.style('opacity', highlightedNodeIds.has(id) || id === selectedNodeId ? 1 : 0.25);
			} else {
				ele.style('opacity', 1);
			}
		});

		cy.edges().forEach((ele) => {
			if (tracing) {
				const src = parseInt(ele.source().id().slice(1));
				const tgt = parseInt(ele.target().id().slice(1));
				// Highlight edges where a prev holder transmitted to a new holder
				const isTransmission =
					(opTrace!.prevHolders.has(src) && opTrace!.newHolders.has(tgt)) ||
					(opTrace!.prevHolders.has(tgt) && opTrace!.newHolders.has(src));
				const isBetweenHolders = opTrace!.holders.has(src) && opTrace!.holders.has(tgt);

				if (isTransmission) {
					ele.style('line-color', '#40e0a0');
					ele.style('target-arrow-color', '#40e0a0');
					ele.style('width', 2.5);
					ele.style('opacity', 1);
				} else if (isBetweenHolders) {
					ele.style('line-color', '#4a90d9');
					ele.style('target-arrow-color', '#4a90d9');
					ele.style('width', 1.5);
					ele.style('opacity', 0.6);
				} else {
					ele.style('line-color', '#444');
					ele.style('target-arrow-color', '#444');
					ele.style('width', 1);
					ele.style('opacity', 0.1);
				}
			} else if (hasHighlight) {
				ele.style('line-color', '#444');
				ele.style('target-arrow-color', '#444');
				ele.style('width', 1);
				ele.style('opacity', 0.15);
			} else {
				ele.style('line-color', '#444');
				ele.style('target-arrow-color', '#444');
				ele.style('width', 1);
				ele.style('opacity', 1);
			}
		});
	}

	onMount(() => {
		cy = cytoscape({
			container,
			elements: snapshotToElements(snapshot),
			style: [
				{
					selector: 'node',
					style: {
						label: 'data(label)',
						'font-size': 8,
						color: '#ccc',
						'text-valign': 'center',
						'text-halign': 'center',
						width: 20,
						height: 20,
						'background-color': '#555'
					}
				},
				{
					selector: 'edge',
					style: {
						width: 1,
						'line-color': '#444',
						'target-arrow-color': '#444',
						'target-arrow-shape': 'triangle',
						'curve-style': 'bezier',
						'arrow-scale': 0.6
					}
				}
			],
			layout: { name: 'cose', animate: false, randomize: true },
			userZoomingEnabled: true,
			userPanningEnabled: true
		});

		cy.on('tap', 'node', (evt) => {
			const id = parseInt(evt.target.id().slice(1));
			onselect(id);
		});

		cy.on('tap', (evt) => {
			if (evt.target === cy) {
				onselect(null);
			}
		});

		layoutDone = true;
		updateNodeStyles(snapshot);

		return () => {
			cy?.destroy();
		};
	});

	$effect(() => {
		snapshot;
		selectedNodeId;
		highlightedNodeIds;
		opTrace;
		if (cy && layoutDone) {
			updateNodeStyles(snapshot);
		}
	});
</script>

<div class="graph" bind:this={container}></div>

<style>
	.graph {
		width: 100%;
		height: 100%;
		background: #0d1117;
	}
</style>
