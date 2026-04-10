<script lang="ts">
	import type { NodeInfo } from '$lib/types';

	interface Props {
		node: NodeInfo;
		selectedGroupId: number | null;
		selectedOpId: number | null;
		onselectgroup: (groupId: number | null) => void;
		onselectop: (opId: number | null) => void;
		onselectnode: (nodeId: number) => void;
		onclose: () => void;
	}

	let { node, selectedGroupId, selectedOpId, onselectgroup, onselectop, onselectnode, onclose }: Props = $props();

	function toggleGroup(id: number) {
		onselectgroup(selectedGroupId === id ? null : id);
	}

	function toggleOp(id: number) {
		onselectop(selectedOpId === id ? null : id);
	}
</script>

<div class="panel">
	<div class="header">
		<h3>Node {node.id}</h3>
		<button class="close" onclick={onclose}>✕</button>
	</div>
	<table>
		<tbody>
			<tr><td>Ops</td><td>{node.ops.length}</td></tr>
			<tr><td>Contacts</td><td>{node.contacts.length}</td></tr>
			<tr>
				<td>Groups</td>
				<td>
					{#if node.groups.length === 0}
						none
					{:else}
						{#each node.groups as gid, i}
							{#if i > 0}, {/if}
							<button
								class="group-link"
								class:active={selectedGroupId === gid}
								onclick={() => toggleGroup(gid)}
							>{gid}</button>
						{/each}
					{/if}
				</td>
			</tr>
			<tr><td>Neighbors</td><td>{node.neighbors.join(', ') || 'none'}</td></tr>
			<tr><td>Routing ops</td><td>{node.routing_ops.length}</td></tr>
		</tbody>
	</table>

	{#if node.ops.length > 0}
		<h4>Operations</h4>
		<div class="ops-list">
			{#each node.ops as op}
				<div class="op">
					<button
						class="op-link"
						class:active={selectedOpId === op.id}
						onclick={() => toggleOp(op.id)}
					>#{op.id}</button>
					<button class="author-link" onclick={() => onselectnode(op.author)}>from {op.author}</button>
					<span class="op-type">
						{#if op.body.type === 'DirectMessage'}
							DM→{op.body.recipient}
						{:else}
							<button
								class="group-link"
								class:active={selectedGroupId === op.body.group_id}
								onclick={() => toggleGroup(op.body.group_id)}
							>Group {op.body.group_id}</button>
						{/if}
					</span>
					<span class="op-size">{op.payload_size}B</span>
				</div>
			{/each}
		</div>
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
	h4 {
		margin: 12px 0 6px;
		font-size: 13px;
		color: #8ecae6;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12px;
	}
	td {
		padding: 3px 6px;
	}
	.ops-list {
		max-height: 200px;
		overflow-y: auto;
		font-size: 11px;
	}
	.op {
		display: flex;
		gap: 8px;
		padding: 2px 0;
		border-bottom: 1px solid #2a2a3a;
	}
	.op-link {
		background: none;
		border: none;
		color: #8ecae6;
		cursor: pointer;
		padding: 0;
		font-size: inherit;
		text-decoration: underline;
		text-decoration-style: dotted;
		min-width: 30px;
		text-align: left;
	}
	.op-link:hover {
		color: #b8e0f8;
	}
	.op-link.active {
		color: #40e0a0;
		text-decoration-style: solid;
		font-weight: bold;
	}
	.author-link {
		background: none;
		border: none;
		color: #e060e0;
		cursor: pointer;
		padding: 0;
		font-size: inherit;
		text-decoration: underline;
		text-decoration-style: dotted;
	}
	.author-link:hover {
		color: #f090f0;
	}
	.op-type {
		color: #bbb;
	}
	.op-size {
		color: #666;
		margin-left: auto;
	}
	.group-link {
		background: none;
		border: none;
		color: #f0c040;
		cursor: pointer;
		padding: 0;
		font-size: inherit;
		text-decoration: underline;
		text-decoration-style: dotted;
	}
	.group-link:hover {
		color: #ffe080;
	}
	.group-link.active {
		color: #ffe080;
		text-decoration-style: solid;
		font-weight: bold;
	}
</style>
