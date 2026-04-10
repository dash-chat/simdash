export type SimResponse =
	| ({ kind: 'Full' } & SimSnapshot)
	| ({ kind: 'Delta' } & SimDelta);

export interface SimSnapshot {
	seq: number;
	step: number;
	behavior: string;
	available_behaviors: string[];
	graph: GraphStructure;
	nodes: NodeInfo[];
}

export interface SimDelta {
	seq: number;
	prev_seq: number;
	step: number;
	node_deltas: NodeDelta[];
	edges_added: GraphEdge[];
	edges_removed: GraphEdge[];
}

export interface NodeDelta {
	id: number;
	ops_added: OpInfo[];
	ops_removed: number[];
	routing_ops: RoutingOpInfo[];
	stored_size: number;
}

export interface GraphStructure {
	nodes: GraphNode[];
	edges: GraphEdge[];
	subnets: Record<string, number[]>;
}

export interface GraphNode {
	id: number;
}

export interface GraphEdge {
	source: number;
	target: number;
}

export interface NodeInfo {
	id: number;
	ops: OpInfo[];
	contacts: number[];
	groups: number[];
	routing_ops: RoutingOpInfo[];
	neighbors: number[];
	storage_capacity: number | null;
	stored_size: number;
}

export interface OpInfo {
	id: number;
	author: number;
	payload_size: number;
	body: OpBody;
}

export type OpBody =
	| { type: 'GroupMessage'; group_id: number }
	| { type: 'DirectMessage'; recipient: number };

export interface RoutingOpInfo {
	id: number;
	size: number;
	since_last_send: number | null;
}

export interface OpTraceData {
	holders: Set<number>;
	newHolders: Set<number>;
	prevHolders: Set<number>;
}
