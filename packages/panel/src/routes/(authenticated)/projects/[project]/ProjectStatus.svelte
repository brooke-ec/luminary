<script lang="ts">
	import { faArrowsRotate, faBan, faPlay, faRocket, faStop } from "@fortawesome/free-solid-svg-icons";
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import StatusIcon from "$lib/component/StatusIcon.svelte";
	import Tooltip from "$lib/component/Tooltip.svelte";
	import { goto } from "$app/navigation";
	import Fa from "svelte-fa";
	import { api } from "$lib";

	let { project }: { project: api.LuminaryProject } = $props();

	let allAction = $derived.by(() => {
		let services = Object.values(project.services);
		let action = services.pop()?.action;

		if (services.some((service) => service.action !== action)) return "idle";
		return action;
	});

	async function deleteProject() {
		await api.client.DELETE("/api/project/{project}", { params: { path: { project: project.name } } });
		await goto("/projects");
	}
</script>

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
		<PromiseButton fit style="outline" disabled={project.busy} onclick={deleteProject}>
			{#snippet children(loading)}
				<div class="flexr center gap-10">
					{#if !loading}<Fa icon={faBan} />{/if}
					Delete Project
				</div>
			{/snippet}
		</PromiseButton>
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
