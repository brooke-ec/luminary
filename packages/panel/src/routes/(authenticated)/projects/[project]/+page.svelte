<script lang="ts">
	import { faCircleInfo, faClockRotateLeft, faLayerGroup } from "@fortawesome/free-solid-svg-icons";
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import { faSave } from "@fortawesome/free-regular-svg-icons";
	import { getProjects, type LuminaryProject } from "$lib/api";
	import LogTerminal from "$lib/component/LogTerminal.svelte";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import { beforeNavigate, goto } from "$app/navigation";
	import StatusTab from "./ProjectStatus.svelte";
	import EditTabs from "../EditTabs.svelte";
	import { api, isMobile } from "$lib";
	import { page } from "$app/state";
	import { onMount } from "svelte";
	import { watch } from "runed";
	import Fa from "svelte-fa";

	let project = $derived(getProjects()[page.params.project!]);
	let { data } = $props();

	let format = $state(async () => {});

	// svelte-ignore state_referenced_locally
	// @ts-ignore
	let changes: { compose: string; name: string } = $state({});

	function revert() {
		changes.compose = data.compose ?? "";
		changes.name = project.name;
	}

	revert();

	// If the project changes (eg. due to navigation), reset working data to the new project's data
	watch(() => page.params.project, revert);

	// Unsaved check
	let unsaved = $derived.by(() => {
		if (!project) return false;
		return changes.name !== project.name || changes.compose !== data.compose;
	});

	async function save() {
		if (localStorage.getItem("luminary-format-on-save") == "true") await format();

		const rename = changes.name !== project.name;
		const response = await api.client.PATCH(`/api/project/{project}`, {
			params: { path: { project: project.name } },
			body: {
				compose: changes.compose === data.compose ? null : changes.compose,
				to: rename ? changes.name : null,
				creating: false,
			},
		});

		api.putProject(response.data!);
		unsaved = false;

		if (rename) {
			await goto(`/projects/${changes.name}${location.hash}`);
			return;
		}

		data.compose = changes.compose;
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

	beforeNavigate(({ cancel }) => {
		if (unsaved && !confirm("You have unsaved changes. Are you sure you want to leave?")) return cancel();
	});
</script>

<svelte:head>
	<title>{page.params.project} - Luminary</title>
</svelte:head>

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

		<EditTabs bind:format bind:data={changes} tabs={[{ label: "status", icon: faCircleInfo, content: status }]} />
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
				<PromiseButton
					onclick={save}
					disabled={project.name.trim() === "" || !/^[A-Za-z0-9-_]*$/.test(changes.name)}
				>
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
