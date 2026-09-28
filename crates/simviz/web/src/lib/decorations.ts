// The only place VizEvents are interpreted. Pure: events in, decorations out.

import type { EdgeStyle, Field, MoonStyle, NodeId, NodeStyle, VizEvent } from './types';

/** Moon slots per node: slot 0 at the top, the rest clockwise. */
export const MOON_SLOTS = 6;
/** Bounds on a moon's radius as a fraction of its node's radius. */
export const MOON_RADIUS_MIN = 0.1;
export const MOON_RADIUS_MAX = 0.75;

export interface SidebarContent {
	title: string | null;
	fields: Field[];
}

export interface Decorations {
	nodeStyles: Map<NodeId, NodeStyle>;
	edgeStyles: Map<string, EdgeStyle>;
	/** Moons to draw this frame, keyed by `moonKey`. Absent means hidden. */
	moonStyles: Map<string, MoonStyle>;
	sidebar: SidebarContent | null;
}

export function edgeKey(source: NodeId, target: NodeId): string {
	return `${source}->${target}`;
}

export function moonKey(node: NodeId, slot: number): string {
	return `${node}:${slot}`;
}

export function emptyDecorations(): Decorations {
	return { nodeStyles: new Map(), edgeStyles: new Map(), moonStyles: new Map(), sidebar: null };
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
			case 'MoonStyle': {
				if (ev.slot < 0 || ev.slot >= MOON_SLOTS) break;
				const key = moonKey(ev.node, ev.slot);
				const merged = { ...d.moonStyles.get(key), ...ev.style };
				if (merged.visible === false) d.moonStyles.delete(key);
				else d.moonStyles.set(key, merged);
				break;
			}
			case 'Sidebar':
				d.sidebar = { title: ev.title, fields: ev.fields };
				break;
		}
	}
	return d;
}
