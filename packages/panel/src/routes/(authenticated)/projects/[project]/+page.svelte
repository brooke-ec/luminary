<script lang="ts">
	import { faCircleInfo, faClockRotateLeft, faLayerGroup } from "@fortawesome/free-solid-svg-icons";
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import { faSave } from "@fortawesome/free-regular-svg-icons";
	import LogTerminal from "$lib/component/LogTerminal.svelte";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import StatusTab from "./ProjectStatus.svelte";
	import EditTabs from "../EditTabs.svelte";
	import { getProjects } from "$lib/api";
	import { goto } from "$app/navigation";
	import { api, isMobile } from "$lib";
	import { page } from "$app/state";
	import { onMount } from "svelte";
	import Fa from "svelte-fa";

	let project = $derived(getProjects()[page.params.project!]);
	let { data } = $props();

	// svelte-ignore state_referenced_locally
	let copy = $state({
		compose: data.compose ?? "",
		name: project?.name ?? "",
	});

	// Watch for changes to set unsaved state
	let unsaved = $state(false);
	$effect(() => {
		if (!project) return;

		unsaved = copy.name !== project.name || copy.compose !== data.compose;
	});

	function revert() {
		copy.compose = data.compose ?? "";
		copy.name = project.name;
	}

	async function save() {
		const rename = copy.name !== project.name;

		const response = await api.client.PATCH(`/api/project/{project}`, {
			params: { path: { project: project.name } },
			body: {
				compose: copy.compose === data.compose ? null : copy.compose,
				to: rename ? copy.name : null,
				creating: false,
			},
		});

		api.putProject(response.data!);

		if (rename) {
			await goto(`/projects/${copy.name}${location.hash}`);
			return;
		}

		unsaved = false;
		data.compose = copy.compose;
	}

	onMount(() => {
		const saveKeybind = (event: KeyboardEvent) => {
			if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
				event.preventDefault();
				save();
			}
		};

		window.addEventListener("keydown", saveKeybind, true);

		return () => {
			window.removeEventListener("keydown", saveKeybind, true);
		};
	});
</script>

{#if project}
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

		<EditTabs bind:data={copy} tabs={[{ label: "status", icon: faCircleInfo, content: status }]} />
	</div>

	{#snippet status()}
		<StatusTab {project} />
		{#if !isMobile()}
			<h2>Logs</h2>
			<LogTerminal project={project.name} />
		{/if}
	{/snippet}

	{#if unsaved}
		<div style="color: var(--peach); margin-bottom: 10px;">* Unsaved changes</div>
		<div class="flexr gap-10">
			<div>
				<PromiseButton onclick={save} disabled={copy.name.trim() === ""}>
					<div class="flexr gap-5 center">
						<Fa icon={faSave} /> Save
					</div>
				</PromiseButton>
			</div>
			<button class="flexr gap-5 center" onclick={revert}>
				<Fa icon={faClockRotateLeft} /> Revert
			</button>
		</div>
	{/if}
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
