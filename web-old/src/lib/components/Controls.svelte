<script lang="ts">
	import { onDestroy, onMount } from "svelte";

	interface Props {
		step: number;
		loading: boolean;
		behavior: string;
		availableBehaviors: string[];
		onforward: () => void;
		onback: () => void;
		onreset: () => void;
		onbehaviorchange: (behavior: string) => void;
		onspeedchange: (intervalMs: number) => void;
	}

	let {
		step,
		loading,
		behavior,
		availableBehaviors,
		onforward,
		onback,
		onreset,
		onbehaviorchange,
		onspeedchange,
	}: Props = $props();

	let playing = $state(false);
	let sliderValue = $state(1810);
	let intervalMs = $derived(2010 - sliderValue);
	let timer: ReturnType<typeof setInterval> | null = null;

	function play() {
		playing = true;
		startTimer();
	}

	function stop() {
		playing = false;
		stopTimer();
	}

	function startTimer() {
		stopTimer();
		timer = setInterval(() => {
			if (!loading) {
				onforward();
			}
		}, intervalMs);
	}

	function stopTimer() {
		if (timer !== null) {
			clearInterval(timer);
			timer = null;
		}
	}

	$effect(() => {
		intervalMs;
		onspeedchange(intervalMs);
		if (playing) {
			startTimer();
		}
	});

	function handleKeydown(e: KeyboardEvent) {
		if (
			e.code === "Space" &&
			!(
				e.target instanceof HTMLInputElement ||
				e.target instanceof HTMLSelectElement
			)
		) {
			e.preventDefault();
			playing ? stop() : play();
		}
	}

	onMount(() => {
		window.addEventListener("keydown", handleKeydown);
		return () => window.removeEventListener("keydown", handleKeydown);
	});

	onDestroy(() => stopTimer());
</script>

<div class="controls">
	<span class="step">Step {step}</span>

	<div class="button-row">
		<button onclick={onback} disabled={loading || playing || step === 0}
			>←</button
		>
		{#if playing}
			<button class="stop" onclick={stop}>
				<svg
					viewBox="0 0 16 16"
					width="14"
					height="14"
					fill="currentColor"
					><rect x="2" y="2" width="12" height="12" rx="1" /></svg
				>
			</button>
		{:else}
			<button class="play" onclick={play} disabled={loading}>
				<svg
					viewBox="0 0 16 16"
					width="14"
					height="14"
					fill="currentColor"><polygon points="3,1 14,8 3,15" /></svg
				>
			</button>
		{/if}
		<button onclick={onforward} disabled={loading || playing}>→</button>
	</div>

	<div class="speed-row">
		<span class="speed-label">Speed</span>
		<input
			class="speed"
			type="range"
			min="10"
			max="2000"
			step="10"
			bind:value={sliderValue}
		/>
	</div>

	<button
		class="reset"
		onclick={onreset}
		disabled={loading || playing || step === 0}>Reset</button
	>

	<!--
	<div class="behavior-row">
		<span class="behavior-label">Behavior</span>
		<select
			class="behavior-select"
			value={behavior}
			onchange={(e) => { stop(); onbehaviorchange(e.currentTarget.value); }}
			disabled={loading || playing}
		>
			{#each availableBehaviors as b}
				<option value={b}>{b}</option>
			{/each}
		</select>
	</div>
	-->
</div>

<style>
	.controls {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 12px;
	}
	.step {
		font-weight: bold;
		color: #e0e0e0;
		font-size: 14px;
	}
	.button-row {
		display: flex;
		gap: 6px;
	}
	button {
		padding: 6px 14px;
		border: 1px solid #555;
		background: #16213e;
		color: #eee;
		border-radius: 4px;
		cursor: pointer;
		font-size: 13px;
	}
	button:hover:not(:disabled) {
		background: #0f3460;
	}
	button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.play,
	.stop {
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}
	.stop {
		border-color: #a33;
		background: #3a1a1a;
	}
	.stop:hover {
		background: #5a2020;
	}
	.reset {
		font-size: 12px;
		padding: 4px 12px;
		opacity: 0.7;
	}
	.speed-row {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.speed-label {
		color: #888;
		font-size: 11px;
		white-space: nowrap;
	}
	.speed {
		flex: 1;
		accent-color: #0f3460;
	}
	.behavior-row {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.behavior-label {
		color: #888;
		font-size: 11px;
		white-space: nowrap;
	}
	.behavior-select {
		flex: 1;
		padding: 4px 6px;
		border: 1px solid #555;
		background: #16213e;
		color: #eee;
		border-radius: 4px;
		font-size: 12px;
		cursor: pointer;
	}
	.behavior-select:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
