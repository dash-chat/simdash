import type { Command, Frame } from './types';

const BASE = '/api';

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
	const res = await fetch(url, init);
	if (!res.ok) {
		throw new Error(`${res.status}: ${await res.text()}`);
	}
	return res.json();
}

export function getFrame(): Promise<Frame> {
	return fetchJson<Frame>(`${BASE}/frame`);
}

export function send(command: Command): Promise<Frame> {
	return fetchJson<Frame>(`${BASE}/command`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(command)
	});
}
