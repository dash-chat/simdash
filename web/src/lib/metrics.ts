import type { SimSnapshot, OpInfo } from './types';

export interface DeliveryInfo {
	op_id: number;
	author: number;
	body_type: string;
	group_id: number | null;
	delivered: number;
	total_targets: number;
	rate: number;
}

export interface NetworkMetrics {
	total_nodes: number;
	total_edges: number;
	unique_op_ids: number;
	convergence_pct: number;
	avg_ops_per_node: number;
	min_ops_per_node: number;
	max_ops_per_node: number;
	delivery: DeliveryInfo[];
	overall_delivery_rate: number;
}

export function computeMetrics(snapshot: SimSnapshot): NetworkMetrics {
	const { nodes, graph } = snapshot;
	const total_nodes = nodes.length;
	const total_edges = graph.edges.length;

	// Collect all unique ops across the network
	const allOpIds = new Set<number>();
	for (const node of nodes) {
		for (const op of node.ops) {
			allOpIds.add(op.id);
		}
	}
	const unique_op_ids = allOpIds.size;

	// Op counts per node
	const opCounts = nodes.map((n) => n.ops.length);
	const avg_ops_per_node = total_nodes > 0 ? opCounts.reduce((a, b) => a + b, 0) / total_nodes : 0;
	const min_ops_per_node = total_nodes > 0 ? Math.min(...opCounts) : 0;
	const max_ops_per_node = total_nodes > 0 ? Math.max(...opCounts) : 0;

	// Convergence: % of nodes that hold ALL known ops
	const converged = nodes.filter((n) => n.ops.length >= unique_op_ids).length;
	const convergence_pct = total_nodes > 0 ? (converged / total_nodes) * 100 : 0;

	// Build lookup structures for delivery computation
	const nodeOpsSet = new Map<number, Set<number>>();
	for (const node of nodes) {
		nodeOpsSet.set(
			node.id,
			new Set(node.ops.map((op) => op.id))
		);
	}

	const nodeGroups = new Map<number, Set<number>>();
	for (const node of nodes) {
		nodeGroups.set(node.id, new Set(node.groups));
	}

	// Deduplicate ops by id (take the first occurrence)
	const uniqueOps = new Map<number, { op: OpInfo; author_node: number }>();
	for (const node of nodes) {
		for (const op of node.ops) {
			if (!uniqueOps.has(op.id)) {
				uniqueOps.set(op.id, { op, author_node: op.author });
			}
		}
	}

	// Compute per-op delivery
	const delivery: DeliveryInfo[] = [];
	let totalDelivered = 0;
	let totalTargets = 0;

	for (const [opId, { op }] of uniqueOps) {
		let targets: number[];
		let bodyType: string;
		let groupId: number | null = null;

		if (op.body.type === 'DirectMessage') {
			targets = [op.body.recipient];
			bodyType = 'DM';
		} else {
			// GroupMessage: all nodes in that group, excluding author
			groupId = op.body.group_id;
			targets = nodes
				.filter((n) => n.id !== op.author && nodeGroups.get(n.id)?.has(groupId!))
				.map((n) => n.id);
			bodyType = `Group ${groupId}`;
		}

		const delivered = targets.filter((t) => nodeOpsSet.get(t)?.has(opId)).length;

		delivery.push({
			op_id: opId,
			author: op.author,
			body_type: bodyType,
			group_id: groupId,
			delivered,
			total_targets: targets.length,
			rate: targets.length > 0 ? delivered / targets.length : 1
		});

		totalDelivered += delivered;
		totalTargets += targets.length;
	}

	const overall_delivery_rate = totalTargets > 0 ? totalDelivered / totalTargets : 1;

	return {
		total_nodes,
		total_edges,
		unique_op_ids,
		convergence_pct,
		avg_ops_per_node,
		min_ops_per_node,
		max_ops_per_node,
		delivery,
		overall_delivery_rate
	};
}
