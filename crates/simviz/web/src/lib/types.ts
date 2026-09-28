// Wire types mirroring crates/simviz/src/{topology,event}.rs.
// The frontend knows nothing about any particular model: only these shapes.

export type NodeId = number;

export interface Edge {
	source: NodeId;
	target: NodeId;
}

export interface Topology {
	nodes: NodeId[];
	edges: Edge[];
}

export type UiAction =
	| { type: 'SelectNode'; node: NodeId | null }
	| { type: 'Link'; payload: unknown };

export type Command =
	| { type: 'StepForward' }
	| { type: 'StepBack' }
	| { type: 'Reset' }
	| { type: 'Ui'; action: UiAction };

export interface NodeStyle {
	color?: string;
	border_color?: string;
	opacity?: number;
	label?: string;
}

export interface EdgeStyle {
	color?: string;
	width?: number;
	opacity?: number;
}

export type FieldValue =
	| { type: 'Text'; text: string }
	| { type: 'Link'; text: string; action: UiAction }
	| { type: 'List'; items: FieldValue[] };

export interface Field {
	label: string;
	value: FieldValue;
}

export type VizEvent =
	| { type: 'NodeStyle'; node: NodeId; style: NodeStyle }
	| { type: 'EdgeStyle'; source: NodeId; target: NodeId; style: EdgeStyle }
	| { type: 'Sidebar'; title: string | null; fields: Field[] };

export interface Frame {
	step: number;
	topology: Topology;
	selected: NodeId | null;
	events: VizEvent[];
}

/** Frames for steps 0..=n, produced by `Viewer::record` on the Rust side. */
export interface Recording {
	frames: Frame[];
}
