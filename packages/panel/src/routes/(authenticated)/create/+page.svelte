<script lang="ts">
	import { faBan, faCircleInfo, faLayerGroup, faPlus } from "@fortawesome/free-solid-svg-icons";
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import { beforeNavigate, goto } from "$app/navigation";
	import EditTabs from "../projects/EditTabs.svelte";
	import placeholder from "./placeholder.yml?raw";
	import { onMount } from "svelte";
	import { api } from "$lib";
	import Fa from "svelte-fa";

	let format = $state(async () => {});

	// svelte-ignore state_referenced_locally
	let project = $state({
		compose: placeholder,
		name: "unnamed",
	});

	async function create() {
		if (localStorage.getItem("luminary-format-on-save") == "true") await format();

		const response = await api.client.PATCH(`/api/project/{project}`, {
			body: { compose: project.compose, creating: true },
			params: { path: { project: project.name } },
		});

		api.putProject(response.data!);
		await goto(`/projects/${project.name}${location.hash}`);
	}

	onMount(() => {
		const saveKeybind = (event: KeyboardEvent) => {
			if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
				event.preventDefault();
				create();
			}
		};

		window.addEventListener("keydown", saveKeybind, true);

		return () => {
			window.removeEventListener("keydown", saveKeybind, true);
		};
	});

	beforeNavigate(({ cancel }) => {
		if (!confirm("You may have unsaved changes. Are you sure you want to leave?")) cancel();
	});
</script>

<svelte:head>
	<title>Create Project - Luminary</title>
</svelte:head>

{#if project}
	<div class="flexc gap-10">
		<!-- Title Bar -->
		<h1 class="flexr gap-10 center fit">
			<Fa icon={faLayerGroup} size="lg" />
			<div style="display: inline-block;">
				<div style="font-size: 22px;">{project.name}</div>
				<div class="subtext flexr gap-5">New Project</div>
			</div>
		</h1>

		<EditTabs bind:format bind:data={project} />
	</div>

	<div class="flexr gap-10">
		<div>
			<PromiseButton onclick={create} disabled={project.name.trim() === ""}>
				<div class="flexr gap-5 center">
					<Fa icon={faPlus} /> Create
				</div>
			</PromiseButton>
		</div>
		<a class="button flexr gap-5 center" href="../">
			<Fa icon={faBan} /> Cancel
		</a>
	</div>
{:else}
	<div class="flexc gap-10 center">
		<Fa icon={faCircleInfo} size="lg" />
		<div style="font-size: 22px;">Project no longer exists</div>
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
