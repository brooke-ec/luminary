<script lang="ts">
	import { faCircleInfo, faClockRotateLeft, faLayerGroup, faPencil } from "@fortawesome/free-solid-svg-icons";
	import ComposeEditor from "$lib/component/ComposeEditor.svelte";
	import { faSave } from "@fortawesome/free-regular-svg-icons";
	import LogTerminal from "$lib/component/LogTerminal.svelte";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import StatusTab from "./ProjectStatus.svelte";
	import Tabs from "$lib/component/Tabs.svelte";
	import { getProjects } from "$lib/api";
	import { page } from "$app/state";
	import { api, isMobile } from "$lib";
	import Fa from "svelte-fa";

	type Payload = api.components["schemas"]["luminary_node.api.project.ComposeWithName"];

	let project = $derived(getProjects()[page.params.project!]);
	let { data } = $props();

	// svelte-ignore state_referenced_locally
	let payload: Payload = $state({
		name: project.name,
		compose: data.compose,
	});

	// Watch for changes to set unsaved state
	let unsaved = $state(false);
	$effect(() => {
		unsaved = payload.name !== project.name || payload.compose !== data.compose;
	});

	function revert() {
		payload.compose = data.compose;
		payload.name = project.name;
	}

	function save() {
		api.client.PUT(`/api/project/{project}`, {
			params: { path: { project: project.name } },
			body: payload,
		});

		unsaved = false;
		data.compose = payload.compose;
	}
</script>

<div class="flexc gap-10">
	<!-- Title Bar -->
	<h1 class="flexr gap-10 center fit">
		<Fa icon={faLayerGroup} size="lg" />
		<div style="display: inline-block;">
			<div style="font-size: 22px;">{project.name}</div>
			<div class="subtext flexr gap-5">
				<StatusIcon status={project.status} />
				{project.status}
			</div>
		</div>
	</h1>

	<Tabs
		tabs={[
			{ label: "status", icon: faCircleInfo, content: status },
			{ label: "compose", icon: faPencil, content: compose },
		]}
	/>
</div>

{#snippet status()}
	<StatusTab {project} />
	{#if !isMobile()}
		<h2>Logs</h2>
		<LogTerminal project={project.name} />
	{/if}
{/snippet}

{#snippet compose()}
	<div>
		<label for="name">Name</label>
		<input required id="name" type="text" bind:value={payload.name} />
	</div>

	<h2>Compose</h2>
	<ComposeEditor bind:content={payload.compose} />
{/snippet}

{#if unsaved}
	<div style="color: var(--peach); margin-bottom: 10px;">* Unsaved changes</div>
	<div class="flexr gap-10">
		<button class="flexr gap-5 center" onclick={save}>
			<Fa icon={faSave} /> Save
		</button>
		<button class="flexr gap-5 center" onclick={revert}>
			<Fa icon={faClockRotateLeft} /> Revert
		</button>
	</div>
{/if}

<style lang="scss">
	// Modify h2 of all child components
	* :global(h2) {
		margin-bottom: 5px;
		font-size: 16px;

		&:not(:first-child) {
			margin-top: 15px;
		}
	}
</style>
