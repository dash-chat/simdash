import type { SimSnapshot, StepDelta } from './types';

const BASE = '/api';

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
	const res = await fetch(url, init);
	if (!res.ok) {
		const text = await res.text();
		throw new Error(`${res.status}: ${text}`);
	}
	return res.json();
}

export function getState(): Promise<SimSnapshot> {
	return fetchJson(`${BASE}/state`);
}

export function stepForward(): Promise<StepDelta> {
	return fetchJson(`${BASE}/step/forward`, { method: 'POST' });
}

export function stepBack(): Promise<SimSnapshot> {
	return fetchJson(`${BASE}/step/back`, { method: 'POST' });
}

export function reset(): Promise<SimSnapshot> {
	return fetchJson(`${BASE}/reset`, { method: 'POST' });
}
