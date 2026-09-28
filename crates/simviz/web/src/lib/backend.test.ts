import { describe, expect, it } from 'vitest';
import { RecordedBackend } from './backend';
import type { Frame, Recording } from './types';

function frame(step: number): Frame {
	return {
		step,
		topology: { nodes: [0, 1], edges: step > 0 ? [{ source: 0, target: 1 }] : [] },
		selected: null,
		events: []
	};
}

const recording: Recording = { frames: [frame(0), frame(1), frame(2)] };

describe('RecordedBackend', () => {
	it('starts at the first frame', async () => {
		const b = new RecordedBackend(recording);
		expect((await b.frame()).step).toBe(0);
	});

	it('steps forward and back within the recording, clamping at both ends', async () => {
		const b = new RecordedBackend(recording);
		expect((await b.send({ type: 'StepBack' })).step).toBe(0);
		expect((await b.send({ type: 'StepForward' })).step).toBe(1);
		expect((await b.send({ type: 'StepForward' })).step).toBe(2);
		expect((await b.send({ type: 'StepForward' })).step).toBe(2);
		expect((await b.send({ type: 'StepBack' })).step).toBe(1);
	});

	it('tracks selection locally and clears it on reset', async () => {
		const b = new RecordedBackend(recording);
		const f = await b.send({ type: 'Ui', action: { type: 'SelectNode', node: 1 } });
		expect(f.selected).toBe(1);
		expect((await b.send({ type: 'StepForward' })).selected).toBe(1);
		expect((await b.send({ type: 'Reset' })).selected).toBeNull();
		expect((await b.frame()).step).toBe(0);
	});

	it('ignores links', async () => {
		const b = new RecordedBackend(recording);
		const f = await b.send({ type: 'Ui', action: { type: 'Link', payload: { x: 1 } } });
		expect(f.step).toBe(0);
	});

	it('does not mutate the recorded frames', async () => {
		const b = new RecordedBackend(recording);
		await b.send({ type: 'Ui', action: { type: 'SelectNode', node: 0 } });
		expect(recording.frames[0].selected).toBeNull();
	});

	it('rejects an empty recording', () => {
		expect(() => new RecordedBackend({ frames: [] })).toThrow();
	});
});
