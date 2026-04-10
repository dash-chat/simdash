<script lang="ts">
	import type { NetworkMetrics } from '$lib/metrics';

	interface Props {
		metrics: NetworkMetrics;
		selectedGroupId: number | null;
		selectedOpId: number | null;
		onselectgroup: (groupId: number | null) => void;
		onselectop: (opId: number | null) => void;
		onselectnode: (nodeId: number) => void;
	}

	let { metrics, selectedGroupId, selectedOpId, onselectgroup, onselectop, onselectnode }: Props = $props();

	function toggleGroup(id: number) {
		onselectgroup(selectedGroupId === id ? null : id);
	}

	function toggleOp(id: number) {
		onselectop(selectedOpId === id ? null : id);
	}
</script>

<div class="panel">
	<h3>Network Metrics</h3>
	<table>
		<tbody>
			<tr><td>Nodes</td><td>{metrics.total_nodes}</td></tr>
			<tr><td>Edges</td><td>{metrics.total_edges}</td></tr>
			<tr><td>Unique ops</td><td>{metrics.unique_op_ids}</td></tr>
			<tr><td>Convergence</td><td>{metrics.convergence_pct.toFixed(1)}%</td></tr>
			<tr><td>Avg ops/node</td><td>{metrics.avg_ops_per_node.toFixed(1)}</td></tr>
			<tr><td>Min ops/node</td><td>{metrics.min_ops_per_node}</td></tr>
			<tr><td>Max ops/node</td><td>{metrics.max_ops_per_node}</td></tr>
			<tr><td>Delivery rate</td><td>{(metrics.overall_delivery_rate * 100).toFixed(1)}%</td></tr>
		</tbody>
	</table>

	{#if metrics.delivery.length > 0}
		<h4>Per-Op Delivery</h4>
		<div class="delivery-table">
			<table>
				<thead>
					<tr>
						<th>Op</th>
						<th>Author</th>
						<th>Type</th>
						<th>Delivered</th>
						<th>Rate</th>
					</tr>
				</thead>
				<tbody>
					{#each metrics.delivery as d}
						<tr>
							<td>
								<button
									class="op-link"
									class:active={selectedOpId === d.op_id}
									onclick={() => toggleOp(d.op_id)}
								>{d.op_id}</button>
							</td>
							<td><button class="author-link" onclick={() => onselectnode(d.author)}>{d.author}</button></td>
							<td>
								{#if d.group_id !== null}
									<button
										class="group-link"
										class:active={selectedGroupId === d.group_id}
										onclick={() => toggleGroup(d.group_id!)}
									>Group {d.group_id}</button>
								{:else}
									{d.body_type}
								{/if}
							</td>
							<td>{d.delivered}/{d.total_targets}</td>
							<td>{(d.rate * 100).toFixed(0)}%</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.panel {
		padding: 12px;
		color: #e0e0e0;
	}
	h3 {
		margin: 0 0 8px;
		font-size: 15px;
		color: #8ecae6;
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
	td, th {
		padding: 3px 6px;
		text-align: left;
	}
	th {
		color: #999;
		border-bottom: 1px solid #444;
	}
	tbody tr:hover {
		background: #1a1a2e;
	}
	.delivery-table {
		max-height: 200px;
		overflow-y: auto;
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
