<script lang="ts">
	import { faCircleInfo, faGears, faLayerGroup, faPencil } from "@fortawesome/free-solid-svg-icons";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import Tabs from "$lib/component/Tabs.svelte";
	import StatusTab from "./StatusTab.svelte";
	import { getList } from "$lib/api";
	import { page } from "$app/state";
	import Fa from "svelte-fa";

	let project = $derived(getList()[page.params.project!]);
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
			{ label: "variables", icon: faGears, content: variables },
		]}
	/>
</div>

{#snippet status()}
	<StatusTab {project} />
{/snippet}

{#snippet compose()}
	<h2>Compose Tab</h2>
{/snippet}

{#snippet variables()}
	<h2>Variables Tab</h2>
{/snippet}
