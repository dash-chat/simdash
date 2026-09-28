import { describe, expect, it } from 'vitest';
import { applyEvents, edgeKey, moonKey } from './decorations';
import type { VizEvent } from './types';

describe('applyEvents', () => {
	it('merges later styles for the same target over earlier ones', () => {
		const d = applyEvents([
			{ type: 'NodeStyle', node: 1, style: { color: '#f00', label: 'a' } },
			{ type: 'NodeStyle', node: 1, style: { color: '#0f0' } },
			{ type: 'EdgeStyle', source: 1, target: 2, style: { width: 3 } }
		]);
		expect(d.nodeStyles.get(1)).toEqual({ color: '#0f0', label: 'a' });
		expect(d.edgeStyles.get(edgeKey(1, 2))).toEqual({ width: 3 });
	});

	it('shows a moon only when styled, and hides it on visible: false', () => {
		const show: VizEvent = { type: 'MoonStyle', node: 3, slot: 2, style: { color: '#00f', label: '7' } };
		let d = applyEvents([show]);
		expect(d.moonStyles.get(moonKey(3, 2))).toEqual({ color: '#00f', label: '7' });
		expect(d.moonStyles.size).toBe(1);

		d = applyEvents([show, { type: 'MoonStyle', node: 3, slot: 2, style: { visible: false } }]);
		expect(d.moonStyles.size).toBe(0);
	});

	it('ignores moon slots out of range', () => {
		const d = applyEvents([
			{ type: 'MoonStyle', node: 3, slot: 6, style: {} },
			{ type: 'MoonStyle', node: 3, slot: -1, style: {} }
		]);
		expect(d.moonStyles.size).toBe(0);
	});
});
