<script lang="ts">
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import Tooltip from "$lib/component/Tooltip.svelte";
	import { goto } from "$app/navigation";
	import { api, closeDialog, openDialog } from "$lib";
	import Fa from "svelte-fa";
	import {
		faArrowsRotate,
		faBan,
		faCircleExclamation,
		faDownload,
		faHammer,
		faPlay,
		faRocket,
		faStop,
		faTimeline,
	} from "@fortawesome/free-solid-svg-icons";

	let { project }: { project: api.LuminaryProject } = $props();
	let confirmation = $state("");

	let allAction = $derived.by(() => {
		let services = Object.values(project.services);
		let action = services.pop()?.action;

		if (services.some((service) => service.action !== action)) return "idle";
		return action;
	});

	function clickDelete() {
		openDialog({ title: deletionTitle, content: deletionContent, parameters: project.name });
		confirmation = "";
	}

	async function deleteProject() {
		await api.client.DELETE("/api/project/{project}", { params: { path: { project: project.name } } });
		await goto("/projects");
		closeDialog();
	}
</script>

{#snippet deletionTitle()}
	<span style:color="var(--red)" style="padding-right: 5px;">
		<Fa icon={faCircleExclamation} />
	</span>
	<span>Delete Project</span>
{/snippet}

{#snippet deletionContent(name: string)}
	<p>
		Are you sure that you want to delete <span style="font-weight: bold">{name}</span>?
		<br />
		This will delete the entire project directory and its contents.
	</p>
	<p style="color: var(--red); font-weight: bold; text-align: center">THIS ACTION IS IRREVERSIBLE</p>
	<br />
	<p>
		<label for="confirmation">
			Enter <span style="color: var(--red); font-weight: bold">delete {name}</span> below to confirm deletion:
		</label>
		<input id="confirmation" type="text" bind:value={confirmation} />
	</p>
	<div class="flexr gap-5">
		<PromiseButton
			disabled={project?.busy || confirmation !== `delete ${project.name}`}
			onclick={deleteProject}
			style="danger"
			fit
		>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faCircleExclamation} /> Delete Project
					{:else}
						Deleting...
					{/if}
				</div>
			{/snippet}
		</PromiseButton>
		<button class="outline" onclick={closeDialog}>
			<div class="flexr center gap-10">
				<Fa icon={faBan} /> Cancel
			</div>
		</button>
	</div>
{/snippet}

<h2>Actions</h2>
{#if project.invalid}
	<div>You must fix the <a href="#compose">compose file</a> to trigger actions.</div>
{:else}
	<div class="flexr gap-5 wrap">
		<PromiseButton
			fit
			style="outline"
			disabled={project.busy}
			loading={allAction === "starting"}
			onclick={() =>
				api.client.POST("/api/project/{project}/start", { params: { path: { project: project.name } } })}
		>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faPlay} />{/if}
					Start All
				</div>
			{/snippet}
		</PromiseButton>
		<PromiseButton
			fit
			style="outline"
			disabled={project.busy}
			loading={allAction === "restarting"}
			onclick={() =>
				api.client.POST("/api/project/{project}/restart", { params: { path: { project: project.name } } })}
		>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faArrowsRotate} />{/if}
					Restart All
				</div>
			{/snippet}
		</PromiseButton>
		<PromiseButton
			fit
			style="outline"
			disabled={project.busy}
			loading={allAction === "stopping"}
			onclick={() =>
				api.client.POST("/api/project/{project}/stop", { params: { path: { project: project.name } } })}
		>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faStop} />{/if}
					Stop All
				</div>
			{/snippet}
		</PromiseButton>
		<PromiseButton
			fit
			style="outline"
			disabled={project.busy}
			onclick={() =>
				api.client.POST("/api/project/{project}/recreate", { params: { path: { project: project.name } } })}
		>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faRocket} />{/if}
					Recreate All
				</div>
			{/snippet}
		</PromiseButton>
		<PromiseButton
			fit
			style="outline"
			disabled={project.busy}
			onclick={() =>
				api.client.POST("/api/project/{project}/update", { params: { path: { project: project.name } } })}
		>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faTimeline} />{/if}
					Update All
				</div>
			{/snippet}
		</PromiseButton>
		<button class="outline" disabled={project.busy} onclick={clickDelete}>
			<div class="flexr center gap-10">
				<Fa icon={faBan} />
				Delete Project
			</div>
		</button>
	</div>
{/if}

<h2>Services</h2>

<div class="flexc gap-5">
	{#each Object.values(project.services) as service}
		<div class="service flexr gap-10">
			<div class="flex center" style="width: 30px;">
				<StatusIcon status={service.status} />
			</div>
			<div class="grow">
				<h3>{service.serviceName}</h3>
				<div class="subtext">
					{#if service.orphan}
						<Tooltip
							content={`This service no longer exists in the compose file.
							It will be removed when it is recreated`}
						>
							<span style="color: var(--peach)">orphaned</span>,
						</Tooltip>
					{/if}

					{service.status}
				</div>
			</div>

			{#if !service.orphan}
				<div class="flexr center gap-5">
					<Tooltip placement="left" content="Start Service">
						<PromiseButton
							style="a"
							aria-label="Start Service"
							disabled={service.action !== "idle"}
							loading={service.action === "starting"}
							onclick={() =>
								api.client.POST("/api/project/{project}/service/{service}/start", {
									params: { path: { project: project.name, service: service.serviceName } },
								})}
						>
							{#snippet children(loading)}
								{#if !loading}<Fa icon={faPlay} />{/if}
							{/snippet}
						</PromiseButton>
					</Tooltip>

					<Tooltip placement="left" content="Restart Service">
						<PromiseButton
							style="a"
							aria-label="Restart Service"
							disabled={service.action !== "idle"}
							loading={service.action === "restarting"}
							onclick={() =>
								api.client.POST("/api/project/{project}/service/{service}/restart", {
									params: { path: { project: project.name, service: service.serviceName } },
								})}
						>
							{#snippet children(loading)}
								{#if !loading}<Fa icon={faArrowsRotate} />{/if}
							{/snippet}
						</PromiseButton>
					</Tooltip>

					<Tooltip placement="left" content="Stop Service">
						<PromiseButton
							style="a"
							aria-label="Stop Service"
							disabled={service.action !== "idle"}
							loading={service.action === "stopping"}
							onclick={() =>
								api.client.POST("/api/project/{project}/service/{service}/stop", {
									params: { path: { project: project.name, service: service.serviceName } },
								})}
						>
							{#snippet children(loading)}
								{#if !loading}<Fa icon={faStop} />{/if}
							{/snippet}
						</PromiseButton>
					</Tooltip>

					<Tooltip placement="left" content="Recreate Service">
						<PromiseButton
							style="a"
							aria-label="Recreate Service"
							disabled={service.action !== "idle"}
							onclick={() =>
								api.client.POST("/api/project/{project}/service/{service}/recreate", {
									params: { path: { project: project.name, service: service.serviceName } },
								})}
						>
							{#snippet children(loading)}
								{#if !loading}<Fa icon={faRocket} />{/if}
							{/snippet}
						</PromiseButton>
					</Tooltip>

					<Tooltip placement="left" content="Pull Service">
						<PromiseButton
							style="a"
							aria-label="Pull Service"
							disabled={service.action !== "idle"}
							loading={service.action === "pulling"}
							onclick={() =>
								api.client.POST("/api/project/{project}/service/{service}/pull", {
									params: { path: { project: project.name, service: service.serviceName } },
								})}
						>
							{#snippet children(loading)}
								{#if !loading}<Fa icon={faDownload} />{/if}
							{/snippet}
						</PromiseButton>
					</Tooltip>

					<Tooltip placement="left" content="Rebuild Service">
						<PromiseButton
							style="a"
							aria-label="Rebuild Service"
							disabled={service.action !== "idle"}
							loading={service.action === "building"}
							onclick={() =>
								api.client.POST("/api/project/{project}/service/{service}/build", {
									params: { path: { project: project.name, service: service.serviceName } },
								})}
						>
							{#snippet children(loading)}
								{#if !loading}<Fa icon={faHammer} />{/if}
							{/snippet}
						</PromiseButton>
					</Tooltip>
				</div>
			{/if}
		</div>
	{:else}
		<div>No services found in compose file.</div>
	{/each}
</div>

<style lang="scss">
	h3 {
		font-size: 14px;
	}

	.service {
		border: 1px solid var(--surface2);
		border-radius: 10px;

		padding: 10px;

		max-width: 1000px;
	}
</style>
