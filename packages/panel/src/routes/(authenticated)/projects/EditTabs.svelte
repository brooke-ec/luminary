<script lang="ts">
	import ComposeEditor from "$lib/component/ComposeEditor.svelte";
	import { faPencil } from "@fortawesome/free-solid-svg-icons";
	import Tabs from "$lib/component/Tabs.svelte";
	import type { ComponentProps } from "svelte";

	let {
		tabs = [],
		data = $bindable(),
		format = $bindable(),
	}: {
		tabs?: ComponentProps<typeof Tabs>["tabs"];
		data: { name: string; compose: string };
		format?: () => Promise<void>;
	} = $props();
</script>

<Tabs tabs={[...tabs, { label: "compose", icon: faPencil, content: compose }]} />

{#snippet compose()}
	<div>
		<label for="name"><h2 style="display: inline-block;">Name</h2></label>
		<div style="position: relative; display: flex; align-items: center;">
			{#if data.name.trim() === ""}
				<div class="error">Name is required</div>
			{/if}
			<input required id="name" type="text" bind:value={data.name} />
		</div>
		{#if !/^[A-Za-z0-9-_]*$/.test(data.name)}
			<div style="color: var(--red)">Project name only contain only alphanumeric, underscores and dashes</div>
		{/if}
	</div>

	<h2>Compose</h2>
	<ComposeEditor bind:content={data.compose} bind:format />
{/snippet}

<style lang="scss">
	.error {
		color: var(--red);
		position: absolute;
		margin-left: 10px;
		font-style: italic;
	}
</style>
