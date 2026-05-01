<script lang="ts">
	import { onMount } from 'svelte';
	import cytoscape from 'cytoscape';
	import fcose from 'cytoscape-fcose';
	import type { SimSnapshot, OpTraceData, OpInfo, NodeInfo } from '$lib/types';

	cytoscape.use(fcose);

	interface Props {
		snapshot: SimSnapshot;
		selectedNodeId: number | null;
		selectedAuthorId: number | null;
		highlightedNodeIds: Set<number>;
		opTrace: OpTraceData | null;
		tickIntervalMs: number;
		onselect: (nodeId: number | null) => void;
	}

	let { snapshot, selectedNodeId, selectedAuthorId, highlightedNodeIds, opTrace, tickIntervalMs, onselect }: Props = $props();

	let container: HTMLDivElement;
	let cy: cytoscape.Core | null = null;
	let layoutDone = false;
	let prevGraphRef: object | null = null;

	// Collect all unique ops across the network
	function allUniqueOps(snapshot: SimSnapshot): OpInfo[] {
		const seen = new Map<number, OpInfo>();
		for (const node of snapshot.nodes) {
			for (const op of node.ops) {
				if (!seen.has(op.id)) seen.set(op.id, op);
			}
		}
		return [...seen.values()];
	}

	// Check if an op is "meant for" a given node
	function isOpMeantFor(op: OpInfo, node: NodeInfo): boolean {
		if (op.body.type === 'GroupMessage') {
			return node.groups.includes(op.body.group_id);
		} else {
			return node.id === op.body.recipient;
		}
	}

	function snapshotToElements(snapshot: SimSnapshot): cytoscape.ElementDefinition[] {
		// Build node-to-subnet lookup
		const nodeSubnet = new Map<number, string>();
		for (const [name, ids] of Object.entries(snapshot.graph.subnets)) {
			for (const id of ids) {
				nodeSubnet.set(id, name);
			}
		}

		// Create parent nodes for each subnet
		const subnetParents: cytoscape.ElementDefinition[] = Object.keys(snapshot.graph.subnets).map((name) => ({
			data: { id: `subnet_${name}`, label: name }
		}));

		const nodes = snapshot.graph.nodes.map((n) => {
			const subnet = nodeSubnet.get(n.id);
			return {
				data: {
					id: `n${n.id}`,
					label: `${n.id}`,
					...(subnet ? { parent: `subnet_${subnet}` } : {})
				}
			};
		});

		// Detect bidirectional pairs and deduplicate
		const edgeSet = new Set(snapshot.graph.edges.map((e) => `${e.source}-${e.target}`));
		const seen = new Set<string>();
		const edges: cytoscape.ElementDefinition[] = [];

		snapshot.graph.edges.forEach((e) => {
			const min = Math.min(e.source, e.target);
			const max = Math.max(e.source, e.target);
			const hasReverse = edgeSet.has(`${e.target}-${e.source}`);

			if (hasReverse) {
				const key = `eb${min}-${max}`;
				if (seen.has(key)) return; // skip duplicate of bidirectional pair
				seen.add(key);
				edges.push({
					data: { id: key, source: `n${e.source}`, target: `n${e.target}`, bidir: 'true' }
				});
			} else {
				edges.push({
					data: { id: `e${e.source}-${e.target}`, source: `n${e.source}`, target: `n${e.target}`, bidir: 'false' }
				});
			}
		});

		return [...subnetParents, ...nodes, ...edges];
	}

	let layoutOptions = $derived({
		name: 'fcose',
		animate: true,
		animationDuration: tickIntervalMs,
		quality: 'proof',
		gravity: 1,
		gravityCompound: 1.5,
		gravityRange: 1.5,
		nodeRepulsion: () => 800,
		idealEdgeLength: () => 40,
		edgeElasticity: () => 0.1,
		padding: 10,
	});

	function syncGraph(snapshot: SimSnapshot): boolean {
		if (!cy) return false;
		const newElements = snapshotToElements(snapshot);
		const newIds = new Set(newElements.map((el) => el.data.id as string));

		let edgesChanged = false;

		// Remove elements no longer in the snapshot
		cy.elements().forEach((ele) => {
			if (!newIds.has(ele.id())) {
				if (ele.isEdge()) edgesChanged = true;
				ele.remove();
			}
		});

		// Add elements not yet in cytoscape
		const existingIds = new Set(cy.elements().map((ele) => ele.id()));
		for (const el of newElements) {
			if (!existingIds.has(el.data.id as string)) {
				cy.add(el);
				if (el.data.source) edgesChanged = true;
			}
		}

		return edgesChanged;
	}

	function saturationColor(ratio: number): string {
		// Interpolate from red (0) to green (1)
		const r = Math.round(220 * (1 - ratio));
		const g = Math.round(220 * ratio);
		return `rgb(${r}, ${g}, 60)`;
	}

	function updateNodeStyles(snapshot: SimSnapshot) {
		if (!cy) return;
		const nodeMap = new Map(snapshot.nodes.map((n) => [n.id, n]));
		const hasHighlight = highlightedNodeIds.size > 0;
		const tracing = opTrace !== null;

		// Pre-compute: for each node, how many ops are meant for it and how many it holds
		// Build a map of opId -> Set<nodeId> that should have it
		let meantForCount: Map<number, number> | null = null;
		let heldOfMeantFor: Map<number, number> | null = null;
		if (!tracing) {
			const allOps = allUniqueOps(snapshot);
			meantForCount = new Map();
			heldOfMeantFor = new Map();

			// Build group membership lookup: groupId -> Set<nodeId>
			const groupMembers = new Map<number, Set<number>>();
			for (const node of snapshot.nodes) {
				for (const gid of node.groups) {
					let s = groupMembers.get(gid);
					if (!s) { s = new Set(); groupMembers.set(gid, s); }
					s.add(node.id);
				}
			}

			// Build per-node held op sets
			const nodeHeldOps = new Map<number, Set<number>>();
			for (const node of snapshot.nodes) {
				nodeHeldOps.set(node.id, new Set(node.ops.map((op) => op.id)));
			}

			// For each op, find target nodes and update counts
			for (const op of allOps) {
				let targets: Iterable<number>;
				if (op.body.type === 'DirectMessage') {
					targets = [op.body.recipient];
				} else {
					targets = groupMembers.get(op.body.group_id) ?? [];
				}
				for (const nid of targets) {
					meantForCount.set(nid, (meantForCount.get(nid) ?? 0) + 1);
					if (nodeHeldOps.get(nid)?.has(op.id)) {
						heldOfMeantFor.set(nid, (heldOfMeantFor.get(nid) ?? 0) + 1);
					}
				}
			}
		}

		cy.batch(() => {
		cy!.nodes().not(':parent').forEach((ele) => {
			const id = parseInt(ele.id().slice(1));
			const node = nodeMap.get(id);

			// Background color
			if (tracing) {
				if (opTrace!.newHolders.has(id)) {
					ele.style('background-color', '#40e0a0');
				} else if (opTrace!.holders.has(id)) {
					ele.style('background-color', '#4a90d9');
				} else {
					ele.style('background-color', '#555');
				}
			} else if (node) {
				const total = meantForCount!.get(id) ?? 0;
				if (total === 0) {
					ele.style('background-color', '#bbb');
				} else {
					const held = heldOfMeantFor!.get(id) ?? 0;
					ele.style('background-color', saturationColor(held / total));
				}
			} else {
				ele.style('background-color', '#555');
			}

			// Border
			if (id === selectedNodeId) {
				ele.style('border-width', 3);
				ele.style('border-color', '#fff');
			} else if (id === selectedAuthorId) {
				ele.style('border-width', 3);
				ele.style('border-color', '#e060e0');
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

		cy!.edges().forEach((ele) => {
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
		}); // end cy.batch
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
					selector: ':parent',
					style: {
						'background-color': '#1a2233',
						'background-opacity': 0.6,
						'border-color': '#334',
						'border-width': 1,
						label: 'data(label)',
						'font-size': 10,
						color: '#889',
						'text-valign': 'top',
						'text-halign': 'center',
						'text-margin-y': -6,
						'padding': '10px'
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
				},
				{
					selector: 'edge[bidir="true"]',
					style: {
						'target-arrow-shape': 'none'
					}
				}
			],
			layout: { ...layoutOptions, randomize: true },
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
		selectedAuthorId;
		highlightedNodeIds;
		opTrace;
		if (cy && layoutDone) {
			if (prevGraphRef !== snapshot.graph) {
				const edgesChanged = syncGraph(snapshot);
				prevGraphRef = snapshot.graph;
				if (edgesChanged) {
					cy.layout({ ...layoutOptions, randomize: false }).run();
				}
			}
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
