// Where frames come from. The page talks only to this interface, so the same UI
// runs against a live simviz server or a static recording.

import type { Command, Frame, Recording } from './types';

export interface Backend {
	frame(): Promise<Frame>;
	send(command: Command): Promise<Frame>;
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
	const res = await fetch(url, init);
	if (!res.ok) {
		throw new Error(`${res.status}: ${await res.text()}`);
	}
	return res.json();
}

/** Live simulation served by `simviz::serve`. */
export class HttpBackend implements Backend {
	constructor(private base = '/api') {}

	frame(): Promise<Frame> {
		return fetchJson<Frame>(`${this.base}/frame`);
	}

	send(command: Command): Promise<Frame> {
		return fetchJson<Frame>(`${this.base}/command`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(command)
		});
	}
}

/**
 * Replays a recording. Stepping clamps to the recorded range. Node selection is
 * tracked locally; presenter links have nowhere to go and are ignored.
 */
export class RecordedBackend implements Backend {
	private index = 0;
	private selected: Frame['selected'] = null;

	constructor(private recording: Recording) {
		if (recording.frames.length === 0) {
			throw new Error('recording has no frames');
		}
	}

	get length(): number {
		return this.recording.frames.length;
	}

	private current(): Frame {
		return { ...this.recording.frames[this.index], selected: this.selected };
	}

	async frame(): Promise<Frame> {
		return this.current();
	}

	async send(command: Command): Promise<Frame> {
		switch (command.type) {
			case 'StepForward':
				this.index = Math.min(this.index + 1, this.length - 1);
				break;
			case 'StepBack':
				this.index = Math.max(this.index - 1, 0);
				break;
			case 'Reset':
				this.index = 0;
				this.selected = null;
				break;
			case 'Ui':
				if (command.action.type === 'SelectNode') {
					this.selected = command.action.node;
				}
				break;
		}
		return this.current();
	}
}

/** Pick a backend from the page URL: `?recording=<url>` replays, otherwise live HTTP. */
export async function backendFromLocation(search: string): Promise<Backend> {
	const url = new URLSearchParams(search).get('recording');
	if (url === null) return new HttpBackend();
	return new RecordedBackend(await fetchJson<Recording>(url));
}
