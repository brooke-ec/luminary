<script lang="ts">
	import StatusIcon, { type LuminaryStatus } from "$lib/component/StatusIcon.svelte";
	import { faMagnifyingGlass, faMinus, faPlus } from "@fortawesome/free-solid-svg-icons";
	import { Accordion } from "melt/builders";
	import { getList } from "$lib/api";
	import { Debounced } from "runed";
	import Fa from "svelte-fa";

	const ORDER = ["healthy", "running", "exited", "paused", "down", "paused"] as LuminaryStatus[];

	const accordion = new Accordion({ multiple: true, value: ORDER });

	let search = $state("");
	const debounced = new Debounced(() => search, 250);

	let groups = $derived(
		Object.entries(
			Object.groupBy(
				Object.values(getList())
					.filter((p) => p.name.includes(debounced.current))
					.toSorted((a, b) => a.name.localeCompare(b.name)),
				(project) => project.status,
			),
		).toSorted(([a], [b]) => ORDER.indexOf(a as LuminaryStatus) - ORDER.indexOf(b as LuminaryStatus)),
	);
</script>

<div class="flexc gap-10 full projects" {...accordion.root}>
	<div class="flexr center gap-10">
		<Fa icon={faMagnifyingGlass} size="lg" />
		<input class="full" type="text" placeholder="Search projects..." bind:value={search} />
	</div>

	{#each groups as [status, projects] (status)}
		{@const item = accordion.getItem({ id: status })}
		<button class="a divider" {...item.trigger} aria-label="toggle {status} projects">
			<Fa icon={item.isExpanded ? faMinus : faPlus} />
			<StatusIcon status={status as LuminaryStatus} />
			<div {...item.heading}>{status} ({Object.keys(projects).length})</div>
			<hr />
		</button>
		{#if item.isExpanded}
			<div class="grid" {...item.content}>
				{#each projects as project (project.name)}
					<a href="/project/{project.name}" class="project">
						<h2>
							<StatusIcon status={project.status} />
							{project.name}
						</h2>
						<div style="color: var(--subtext0);">
							{Object.keys(project.services).length} services {project.status}
						</div>
					</a>
				{/each}
			</div>
		{/if}
	{/each}

	{#if debounced.current}
		<button
			class="a"
			onclick={() => {
				search = "";
				debounced.setImmediately("");
			}}
		>
			Clear search filter
		</button>
	{/if}
</div>

<style lang="scss">
	.projects {
		container: projects / inline-size;
	}

	.divider {
		display: flex;
		align-items: center;
		gap: 10px;

		color: var(--text);
		text-decoration: none !important;

		hr {
			border-color: var(--subtext0);
			flex-grow: 1;
		}
	}

	.grid {
		grid-template-columns: repeat(3, minmax(0, 1fr));
		display: grid;
		gap: 10px;

		width: 100%;
	}

	@container projects (max-width: 919px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}

	.project {
		transition: background-color 250ms ease;
		background-color: var(--surface0);
		border-radius: 10px;
		padding: 10px;

		color: inherit;

		&:hover {
			background-color: var(--surface1);
			text-decoration: none;
		}

		h2 {
			display: flex;
			align-items: center;
			gap: 10px;

			margin-bottom: 5px;
		}
	}
</style>
