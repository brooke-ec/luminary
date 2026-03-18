<script lang="ts">
	import { faCircleInfo, faLayerGroup, faPencil } from "@fortawesome/free-solid-svg-icons";
	import ComposeEditor from "$lib/component/ComposeEditor.svelte";
	import LogTerminal from "$lib/component/LogTerminal.svelte";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import Tabs from "$lib/component/Tabs.svelte";
	import StatusTab from "./ProjectStatus.svelte";
	import { getProjects } from "$lib/api";
	import { page } from "$app/state";
	import { isMobile } from "$lib";
	import Fa from "svelte-fa";

	let project = $derived(getProjects()[page.params.project!]);
	let { data } = $props();
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
		<input required id="name" type="text" value={project.name} />
	</div>

	<h2>Compose</h2>
	<ComposeEditor bind:content={data.compose} />
{/snippet}

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
