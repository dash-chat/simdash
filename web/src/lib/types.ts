export interface SimSnapshot {
	step: number;
	graph: GraphStructure;
	nodes: NodeInfo[];
}

export interface GraphStructure {
	nodes: GraphNode[];
	edges: GraphEdge[];
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

export interface StepDelta {
	step: number;
	node_diffs: NodeDelta[];
}

export interface NodeDelta {
	node_id: number;
	new_ops: OpInfo[];
}

export interface OpTraceData {
	holders: Set<number>;
	newHolders: Set<number>;
	prevHolders: Set<number>;
}
