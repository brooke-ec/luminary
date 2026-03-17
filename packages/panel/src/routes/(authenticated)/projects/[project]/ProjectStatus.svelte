<script lang="ts">
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import { faArrowsRotate, faPlay, faRocket, faStop } from "@fortawesome/free-solid-svg-icons";
	import Fa from "svelte-fa";
	import { api } from "$lib";
	import StatusIcon from "$lib/component/StatusIcon.svelte";

	let { project }: { project: api.LuminaryProject } = $props();

	let allAction = $derived.by(() => {
		let services = Object.values(project.services);
		let action = services.pop()?.action;

		if (services.some((service) => service.action !== action)) return "idle";
		return action;
	});
</script>

<h2>Actions</h2>
<div class="flexr gap-5 wrap">
	<div>
		<PromiseButton
			style="outline"
			disabled={project.busy}
			loading={allAction === "starting"}
			onclick={() =>
				api.client.POST("/api/project/{project}/start", { params: { path: { project: project.name } } })}
		>
			<div class="flexr center gap-10">
				<Fa icon={faPlay} />
				Start All
			</div>
		</PromiseButton>
	</div>
	<div>
		<PromiseButton
			style="outline"
			disabled={project.busy}
			loading={allAction === "restarting"}
			onclick={() =>
				api.client.POST("/api/project/{project}/restart", { params: { path: { project: project.name } } })}
		>
			<div class="flexr center gap-10">
				<Fa icon={faArrowsRotate} />
				Restart All
			</div>
		</PromiseButton>
	</div>
	<div>
		<PromiseButton
			style="outline"
			disabled={project.busy}
			loading={allAction === "stopping"}
			onclick={() =>
				api.client.POST("/api/project/{project}/stop", { params: { path: { project: project.name } } })}
		>
			<div class="flexr center gap-10">
				<Fa icon={faStop} />
				Stop All
			</div>
		</PromiseButton>
	</div>
	<div>
		<PromiseButton
			style="outline"
			disabled={project.busy}
			onclick={() =>
				api.client.POST("/api/project/{project}/redeploy", { params: { path: { project: project.name } } })}
		>
			<div class="flexr center gap-10">
				<Fa icon={faRocket} />
				Redeploy All
			</div>
		</PromiseButton>
	</div>
</div>

<h2>Services</h2>

<div class="flexc gap-5">
	{#each Object.values(project.services) as service}
		<div class="service flexr gap-10">
			<div class="flex center" style="width: 30px;">
				<StatusIcon status={service.status} />
			</div>
			<div class="grow">
				<h3>{service.serviceName}</h3>
				<div class="subtext">{service.status}</div>
			</div>

			<div class="flexr center gap-5">
				<PromiseButton
					style="a"
					disabled={service.action !== "idle"}
					loading={service.action === "starting"}
					onclick={() =>
						api.client.POST("/api/project/{project}/service/{service}/start", {
							params: { path: { project: project.name, service: service.serviceName } },
						})}
				>
					<div class="flexr center gap-10">
						<Fa icon={faPlay} />
					</div>
				</PromiseButton>
				<PromiseButton
					style="a"
					disabled={service.action !== "idle"}
					loading={service.action === "restarting"}
					onclick={() =>
						api.client.POST("/api/project/{project}/service/{service}/restart", {
							params: { path: { project: project.name, service: service.serviceName } },
						})}
				>
					<div class="flexr center gap-10">
						<Fa icon={faArrowsRotate} />
					</div>
				</PromiseButton>

				<PromiseButton
					style="a"
					disabled={service.action !== "idle"}
					loading={service.action === "stopping"}
					onclick={() =>
						api.client.POST("/api/project/{project}/service/{service}/stop", {
							params: { path: { project: project.name, service: service.serviceName } },
						})}
				>
					<div class="flexr center gap-10">
						<Fa icon={faStop} />
					</div>
				</PromiseButton>

				<PromiseButton
					style="a"
					disabled={service.action !== "idle"}
					onclick={() =>
						api.client.POST("/api/project/{project}/service/{service}/redeploy", {
							params: { path: { project: project.name, service: service.serviceName } },
						})}
				>
					<div class="flexr center gap-10">
						<Fa icon={faRocket} />
					</div>
				</PromiseButton>
			</div>
		</div>
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
