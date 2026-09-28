<script lang="ts">
	import { onMount } from 'svelte';
	import cytoscape from 'cytoscape';
	import fcose from 'cytoscape-fcose';
	import type { NodeId, Topology } from '$lib/types';
	import { edgeKey, MOON_RADIUS_MAX, MOON_RADIUS_MIN, MOON_SLOTS, type Decorations } from '$lib/decorations';

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
	const DEFAULT_MOON = { color: '#8b949e', border_color: '#0d1117', opacity: 1, radius: 0.35 };
	const NODE_SIZE = 20;
	// Wheel zoom: factor per pixel of scroll, eased towards the target each frame.
	const WHEEL_ZOOM_RATE = 0.0015;
	const WHEEL_ZOOM_EASE = 0.35;

	let container: HTMLDivElement;
	let cy: cytoscape.Core | null = null;
	let prevTopology: Topology | null = null;

	function nodeElId(id: NodeId): string {
		return `n${id}`;
	}
	function parseNodeId(elId: string): NodeId {
		return parseInt(elId.slice(1));
	}
	function moonElId(node: NodeId, slot: number): string {
		return `m${node}-${slot}`;
	}

	/** A moon's radius in pixels from its style fraction. */
	function moonRadius(fraction: number): number {
		const f = Math.min(MOON_RADIUS_MAX, Math.max(MOON_RADIUS_MIN, fraction));
		return (f * NODE_SIZE) / 2;
	}

	/** Where `slot` sits around a node at `center`: slot 0 on top, then clockwise, just outside the node. */
	function moonPosition(center: cytoscape.Position, slot: number, radius: number): cytoscape.Position {
		const angle = -Math.PI / 2 + (slot * 2 * Math.PI) / MOON_SLOTS;
		const orbit = NODE_SIZE / 2 + radius + 2;
		return { x: center.x + orbit * Math.cos(angle), y: center.y + orbit * Math.sin(angle) };
	}

	/** Move a node's moons along with it. */
	function placeMoons(node: cytoscape.NodeSingular) {
		node.cy().nodes(`[kind = "moon"][planet = "${node.id()}"]`).forEach((moon) => {
			moon.position(moonPosition(node.position(), moon.data('slot'), moon.data('radius')));
		});
	}

	// Bidirectional pairs collapse into one undirected element so the layout doesn't double-count them.
	function toElements(t: Topology): cytoscape.ElementDefinition[] {
		const present = new Set(t.edges.map((e) => edgeKey(e.source, e.target)));
		const seen = new Set<string>();
		const nodes = t.nodes.map((id) => ({ data: { id: nodeElId(id), kind: 'planet', label: `${id}` } }));
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
					kind: 'edge',
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
		cy.elements('[kind != "moon"]').forEach((el) => {
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
			cy!.nodes('[kind = "planet"]').forEach((el) => {
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
				// keys[0] runs source→target; a collapsed bidirectional pair also has keys[1] the other way.
				const keys: string[] = el.data('keys') ?? [];
				const fwd = decorations.edgeStyles.get(keys[0]);
				const back = keys.length > 1 ? decorations.edgeStyles.get(keys[1]) : undefined;
				const s = { ...DEFAULT_EDGE, ...fwd, ...back };
				el.style('width', Math.max(fwd?.width ?? 0, back?.width ?? 0) || s.width);
				el.style('opacity', s.opacity);
				if (el.data('bidir') === 'true') {
					// Undirected link: arrows show which way styled traffic runs,
					// and a two-way link blends the colours along its length.
					el.style('target-arrow-shape', fwd ? 'triangle' : 'none');
					el.style('source-arrow-shape', back ? 'triangle' : 'none');
					el.style('target-arrow-color', fwd?.color ?? s.color);
					el.style('source-arrow-color', back?.color ?? s.color);
					if (fwd?.color && back?.color && fwd.color !== back.color) {
						el.style('line-fill', 'linear-gradient');
						el.style('line-gradient-stop-colors', `${back.color} ${fwd.color}`);
						el.style('line-gradient-stop-positions', '0% 100%');
					} else {
						el.style('line-fill', 'solid');
						el.style('line-color', fwd?.color ?? back?.color ?? s.color);
					}
				} else {
					el.style('line-color', s.color);
					el.style('target-arrow-color', s.color);
				}
			});
			// Moons exist only while styled: add the ones this frame wants,
			// drop the rest.
			const wanted = new Set<string>();
			for (const [key, style] of decorations.moonStyles) {
				const [node, slot] = key.split(':').map(Number);
				const planet = cy!.getElementById(nodeElId(node));
				if (planet.empty()) continue;
				const id = moonElId(node, slot);
				wanted.add(id);
				const s = { ...DEFAULT_MOON, ...style };
				const radius = moonRadius(s.radius);
				let moon = cy!.getElementById(id);
				if (moon.empty()) {
					moon = cy!.add({
						data: { id, kind: 'moon', planet: planet.id(), slot, radius },
						position: moonPosition(planet.position(), slot, radius),
						grabbable: false,
						selectable: false
					});
				} else if (moon.data('radius') !== radius) {
					moon.data('radius', radius);
					moon.position(moonPosition(planet.position(), slot, radius));
				}
				moon.style('width', radius * 2);
				moon.style('height', radius * 2);
				moon.style('font-size', Math.max(4, Math.round(radius * 1.4)));
				// A ratio turns the moon into a pie: the wedge in the moon's
				// colour over a `background` disc (the border colour when
				// unset, so the rest looks hollow).
				if (s.ratio === undefined) {
					moon.style('background-color', s.color);
					moon.style('pie-1-background-size', 0);
				} else {
					const wedge = Math.min(1, Math.max(0, s.ratio)) * 100;
					moon.style('background-color', s.background ?? s.border_color);
					moon.style('pie-1-background-color', s.color);
					moon.style('pie-1-background-size', wedge);
				}
				moon.style('border-color', s.border_color);
				moon.style('opacity', s.opacity);
				moon.style('label', s.label ?? '');
			}
			cy!.nodes('[kind = "moon"]').forEach((moon) => {
				if (!wanted.has(moon.id())) moon.remove();
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
						width: NODE_SIZE,
						height: NODE_SIZE,
						'background-color': DEFAULT_NODE.color
					}
				},
				{
					selector: 'node[kind = "moon"]',
					style: {
						label: '',
						'font-size': 5,
						color: '#eee',
						'border-width': 1,
						'border-color': DEFAULT_MOON.border_color,
						'background-color': DEFAULT_MOON.color,
						'pie-size': '100%',
						'z-index': 10,
						events: 'yes'
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
			// Wheel zoom is handled below: cytoscape's own steps are too coarse.
			userZoomingEnabled: false,
			userPanningEnabled: true
		});
		prevTopology = topology;

		let zoomTarget = cy.zoom();
		let zoomAnchor: cytoscape.Position = { x: 0, y: 0 };
		let zoomRaf = 0;
		const easeZoom = () => {
			if (!cy) return;
			const z = cy.zoom();
			const next = Math.abs(zoomTarget - z) < 1e-3 ? zoomTarget : z + (zoomTarget - z) * WHEEL_ZOOM_EASE;
			cy.zoom({ level: next, renderedPosition: zoomAnchor });
			zoomRaf = next === zoomTarget ? 0 : requestAnimationFrame(easeZoom);
		};
		const onWheel = (e: WheelEvent) => {
			if (!cy) return;
			e.preventDefault();
			if (!zoomRaf) zoomTarget = cy.zoom();
			const pixels = e.deltaY * (e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? 100 : 1);
			zoomTarget = Math.min(cy.maxZoom(), Math.max(cy.minZoom(), zoomTarget * Math.exp(-pixels * WHEEL_ZOOM_RATE)));
			zoomAnchor = { x: e.offsetX, y: e.offsetY };
			if (!zoomRaf) zoomRaf = requestAnimationFrame(easeZoom);
		};
		container.addEventListener('wheel', onWheel, { passive: false });

		cy.on('tap', 'node[kind = "planet"]', (evt) => onselect(parseNodeId(evt.target.id())));
		cy.on('tap', 'node[kind = "moon"]', (evt) => onselect(parseNodeId(evt.target.data('planet'))));
		cy.on('tap', (evt) => {
			if (evt.target === cy) onselect(null);
		});
		// Moons follow their planet through drags and layout animation.
		cy.on('position', 'node[kind = "planet"]', (evt) => placeMoons(evt.target));
		cy.on('layoutstop', () => cy!.nodes('[kind = "planet"]').forEach(placeMoons));

		applyStyles();
		return () => {
			container.removeEventListener('wheel', onWheel);
			if (zoomRaf) cancelAnimationFrame(zoomRaf);
			cy?.destroy();
		};
	});

	$effect(() => {
		topology;
		decorations;
		selected;
		if (!cy) return;
		if (prevTopology !== topology) {
			const edgesChanged = syncTopology(topology);
			prevTopology = topology;
			if (edgesChanged)
				cy
					.elements('[kind != "moon"]')
					.layout({ ...layoutOptions, randomize: false } as cytoscape.LayoutOptions)
					.run();
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
