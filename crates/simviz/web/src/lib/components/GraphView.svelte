<script lang="ts">
	import { onMount } from 'svelte';
	import cytoscape from 'cytoscape';
	import fcose from 'cytoscape-fcose';
	import type { NodeId, Topology } from '$lib/types';
	import { edgeKey, type Decorations } from '$lib/decorations';

	cytoscape.use(fcose);

	interface Props {
		topology: Topology;
		decorations: Decorations;
		selected: NodeId | null;
		tickIntervalMs: number;
		onselect: (node: NodeId | null) => void;
	}

	let { topology, decorations, selected, tickIntervalMs, onselect }: Props = $props();

	const DEFAULT_NODE = { color: '#8b949e', border_color: '#ffffff', opacity: 1 };
	const DEFAULT_EDGE = { color: '#444', width: 1, opacity: 1 };

	let container: HTMLDivElement;
	let cy: cytoscape.Core | null = null;
	let prevTopology: Topology | null = null;

	function nodeElId(id: NodeId): string {
		return `n${id}`;
	}
	function parseNodeId(elId: string): NodeId {
		return parseInt(elId.slice(1));
	}

	// Bidirectional pairs collapse into one undirected element so the layout doesn't double-count them.
	function toElements(t: Topology): cytoscape.ElementDefinition[] {
		const present = new Set(t.edges.map((e) => edgeKey(e.source, e.target)));
		const seen = new Set<string>();
		const nodes = t.nodes.map((id) => ({ data: { id: nodeElId(id), label: `${id}` } }));
		const edges: cytoscape.ElementDefinition[] = [];
		for (const e of t.edges) {
			const bidir = present.has(edgeKey(e.target, e.source));
			const lo = Math.min(e.source, e.target);
			const hi = Math.max(e.source, e.target);
			const id = bidir ? `eb${lo}-${hi}` : `e${e.source}-${e.target}`;
			if (seen.has(id)) continue;
			seen.add(id);
			edges.push({
				data: {
					id,
					source: nodeElId(e.source),
					target: nodeElId(e.target),
					bidir: bidir ? 'true' : 'false',
					// Directed keys this element stands for, for style lookup.
					keys: bidir ? [edgeKey(e.source, e.target), edgeKey(e.target, e.source)] : [edgeKey(e.source, e.target)]
				}
			});
		}
		return [...nodes, ...edges];
	}

	// fcose options are not in cytoscape's typings, so this is loosely typed.
	let layoutOptions: Record<string, unknown> = $derived({
		name: 'fcose',
		animate: true,
		animationDuration: tickIntervalMs,
		quality: 'proof',
		gravity: 1,
		nodeRepulsion: () => 800,
		idealEdgeLength: () => 40,
		edgeElasticity: () => 0.1,
		padding: 10
	});

	/** Diff the topology into cytoscape; returns whether edges changed. */
	function syncTopology(t: Topology): boolean {
		if (!cy) return false;
		const wanted = toElements(t);
		const wantedIds = new Set(wanted.map((el) => el.data.id as string));
		let edgesChanged = false;
		cy.elements().forEach((el) => {
			if (!wantedIds.has(el.id())) {
				if (el.isEdge()) edgesChanged = true;
				el.remove();
			}
		});
		const existing = new Set(cy.elements().map((el) => el.id()));
		for (const el of wanted) {
			if (!existing.has(el.data.id as string)) {
				cy.add(el);
				if (el.data.source) edgesChanged = true;
			}
		}
		return edgesChanged;
	}

	function applyStyles() {
		if (!cy) return;
		cy.batch(() => {
			cy!.nodes().forEach((el) => {
				const id = parseNodeId(el.id());
				const s = { ...DEFAULT_NODE, ...decorations.nodeStyles.get(id) };
				el.style('background-color', s.color);
				el.style('opacity', s.opacity);
				el.style('label', s.label ?? `${id}`);
				if (id === selected) {
					el.style('border-width', 3);
					el.style('border-color', '#fff');
				} else if (decorations.nodeStyles.get(id)?.border_color) {
					el.style('border-width', 3);
					el.style('border-color', s.border_color);
				} else {
					el.style('border-width', 0);
				}
			});
			cy!.edges().forEach((el) => {
				const keys: string[] = el.data('keys') ?? [];
				const override = keys.map((k) => decorations.edgeStyles.get(k)).find((s) => s);
				const s = { ...DEFAULT_EDGE, ...override };
				el.style('line-color', s.color);
				el.style('target-arrow-color', s.color);
				el.style('width', s.width);
				el.style('opacity', s.opacity);
			});
		});
	}

	onMount(() => {
		cy = cytoscape({
			container,
			elements: toElements(topology),
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
						'background-color': DEFAULT_NODE.color
					}
				},
				{
					selector: 'edge',
					style: {
						width: 1,
						'line-color': DEFAULT_EDGE.color,
						'target-arrow-color': DEFAULT_EDGE.color,
						'target-arrow-shape': 'triangle',
						'curve-style': 'bezier',
						'arrow-scale': 0.6
					}
				},
				{ selector: 'edge[bidir="true"]', style: { 'target-arrow-shape': 'none' } }
			],
			layout: { ...layoutOptions, randomize: true } as cytoscape.LayoutOptions,
			userZoomingEnabled: true,
			userPanningEnabled: true
		});
		prevTopology = topology;

		cy.on('tap', 'node', (evt) => onselect(parseNodeId(evt.target.id())));
		cy.on('tap', (evt) => {
			if (evt.target === cy) onselect(null);
		});

		applyStyles();
		return () => cy?.destroy();
	});

	$effect(() => {
		topology;
		decorations;
		selected;
		if (!cy) return;
		if (prevTopology !== topology) {
			const edgesChanged = syncTopology(topology);
			prevTopology = topology;
			if (edgesChanged) cy.layout({ ...layoutOptions, randomize: false } as cytoscape.LayoutOptions).run();
		}
		applyStyles();
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
