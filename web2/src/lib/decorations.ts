// The only place VizEvents are interpreted. Pure: events in, decorations out.

import type { EdgeStyle, Field, NodeId, NodeStyle, VizEvent } from './types';

export interface SidebarContent {
	title: string | null;
	fields: Field[];
}

export interface Decorations {
	nodeStyles: Map<NodeId, NodeStyle>;
	edgeStyles: Map<string, EdgeStyle>;
	sidebar: SidebarContent | null;
}

export function edgeKey(source: NodeId, target: NodeId): string {
	return `${source}->${target}`;
}

export function emptyDecorations(): Decorations {
	return { nodeStyles: new Map(), edgeStyles: new Map(), sidebar: null };
}

/** Fold a frame's events into decorations. Later events for the same target merge over earlier ones. */
export function applyEvents(events: VizEvent[]): Decorations {
	const d = emptyDecorations();
	for (const ev of events) {
		switch (ev.type) {
			case 'NodeStyle':
				d.nodeStyles.set(ev.node, { ...d.nodeStyles.get(ev.node), ...ev.style });
				break;
			case 'EdgeStyle': {
				const key = edgeKey(ev.source, ev.target);
				d.edgeStyles.set(key, { ...d.edgeStyles.get(key), ...ev.style });
				break;
			}
			case 'Sidebar':
				d.sidebar = { title: ev.title, fields: ev.fields };
				break;
		}
	}
	return d;
}
