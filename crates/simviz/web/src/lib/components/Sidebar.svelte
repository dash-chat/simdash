<script lang="ts">
	import type { FieldValue, NodeId, UiAction } from '$lib/types';
	import type { SidebarContent } from '$lib/decorations';

	interface Props {
		selected: NodeId | null;
		content: SidebarContent | null;
		onaction: (action: UiAction) => void;
		onclose: () => void;
	}

	let { selected, content, onaction, onclose }: Props = $props();

	let title = $derived(content?.title ?? (selected !== null ? `Node ${selected}` : ''));
</script>

{#snippet value(v: FieldValue)}
	{#if v.type === 'Text'}
		<span>{v.text}</span>
	{:else if v.type === 'Link'}
		<button class="link" onclick={() => onaction(v.action)}>{v.text}</button>
	{:else if v.type === 'List'}
		{#each v.items as item, i}
			{#if i > 0}, {/if}
			{@render value(item)}
		{/each}
	{/if}
{/snippet}

<div class="panel">
	<div class="header">
		<h3>{title}</h3>
		<button class="close" onclick={onclose}>✕</button>
	</div>
	{#if content && content.fields.length > 0}
		<table>
			<tbody>
				{#each content.fields as field}
					<tr>
						<td class="label">{field.label}</td>
						<td>{@render value(field.value)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{:else}
		<p class="empty">Nothing to show.</p>
	{/if}
</div>

<style>
	.panel {
		padding: 12px;
		color: #e0e0e0;
	}
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 8px;
	}
	h3 {
		margin: 0;
		font-size: 15px;
		color: #8ecae6;
	}
	.close {
		background: none;
		border: none;
		color: #888;
		font-size: 16px;
		cursor: pointer;
		padding: 2px 6px;
		border-radius: 3px;
	}
	.close:hover {
		color: #eee;
		background: #333;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12px;
	}
	td {
		padding: 3px 6px;
		vertical-align: top;
	}
	.label {
		color: #888;
		white-space: nowrap;
	}
	.empty {
		color: #666;
		font-size: 12px;
	}
	.link {
		background: none;
		border: none;
		color: #8ecae6;
		cursor: pointer;
		padding: 0;
		font-size: inherit;
		text-decoration: underline;
		text-decoration-style: dotted;
	}
	.link:hover {
		color: #b8e0f8;
	}
</style>
