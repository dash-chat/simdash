import type { SimSnapshot, SimResponse, SimDelta } from './types';

const BASE = '/api';

let currentSnapshot: SimSnapshot | null = null;

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
	const res = await fetch(url, init);
	if (!res.ok) {
		const text = await res.text();
		throw new Error(`${res.status}: ${text}`);
	}
	return res.json();
}

function applyDelta(base: SimSnapshot, delta: SimDelta): SimSnapshot {
	const nodeMap = new Map(base.nodes.map((n) => [n.id, n]));

	for (const nd of delta.node_deltas) {
		const node = nodeMap.get(nd.id);
		if (!node) continue;

		const removedSet = new Set(nd.ops_removed);
		const newOps = node.ops.filter((op) => !removedSet.has(op.id));
		newOps.push(...nd.ops_added);

		nodeMap.set(nd.id, {
			...node,
			ops: newOps,
			routing_ops: nd.routing_ops,
			stored_size: nd.stored_size
		});
	}

	// Apply graph edge changes
	let graph = base.graph;
	if (delta.edges_added.length > 0 || delta.edges_removed.length > 0) {
		const removedSet = new Set(
			delta.edges_removed.map((e) => `${e.source}-${e.target}`)
		);
		const edges = base.graph.edges.filter(
			(e) => !removedSet.has(`${e.source}-${e.target}`)
		);
		edges.push(...delta.edges_added);
		graph = { ...base.graph, edges };
	}

	return {
		...base,
		seq: delta.seq,
		step: delta.step,
		graph,
		nodes: base.nodes.map((n) => nodeMap.get(n.id) ?? n)
	};
}

async function resolveResponse(resp: SimResponse): Promise<SimSnapshot> {
	if (resp.kind === 'Full') {
		const { kind, ...snapshot } = resp;
		currentSnapshot = snapshot;
		return snapshot;
	}

	// Delta
	const delta = resp as SimDelta & { kind: string };
	if (!currentSnapshot || delta.prev_seq !== currentSnapshot.seq) {
		// Seq mismatch — fall back to full state
		return getState();
	}

	currentSnapshot = applyDelta(currentSnapshot, delta);
	return currentSnapshot;
}

export async function getState(): Promise<SimSnapshot> {
	const resp = await fetchJson<SimResponse>(`${BASE}/state`);
	return resolveResponse(resp);
}

export async function stepForward(): Promise<SimSnapshot> {
	const resp = await fetchJson<SimResponse>(`${BASE}/step/forward`, { method: 'POST' });
	return resolveResponse(resp);
}

export async function stepBack(): Promise<SimSnapshot> {
	const resp = await fetchJson<SimResponse>(`${BASE}/step/back`, { method: 'POST' });
	return resolveResponse(resp);
}

export async function reset(): Promise<SimSnapshot> {
	const resp = await fetchJson<SimResponse>(`${BASE}/reset`, { method: 'POST' });
	return resolveResponse(resp);
}

export async function setBehavior(behavior: string): Promise<SimSnapshot> {
	const resp = await fetchJson<SimResponse>(`${BASE}/behavior`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ behavior })
	});
	return resolveResponse(resp);
}
