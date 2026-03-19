<script lang="ts">
	import ComposeEditor from "$lib/component/ComposeEditor.svelte";
	import { faPencil } from "@fortawesome/free-solid-svg-icons";
	import Tabs from "$lib/component/Tabs.svelte";
	import type { ComponentProps } from "svelte";

	let {
		tabs = [],
		data = $bindable(),
	}: { data: { name: string; compose: string }; tabs?: ComponentProps<typeof Tabs>["tabs"] } = $props();
</script>

<Tabs tabs={[...tabs, { label: "compose", icon: faPencil, content: compose }]} />

{#snippet compose()}
	<div>
		<label for="name">Name</label>
		<input required id="name" type="text" bind:value={data.name} />
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
