<script>
	import Tabs from "$lib/component/Tabs.svelte";
	import { faCircleInfo, faGears, faPencil } from "@fortawesome/free-solid-svg-icons";
</script>

<Tabs
	tabs={[
		{ label: "status", icon: faCircleInfo, content: status },
		{ label: "compose", icon: faPencil, content: compose },
		{ label: "variables", icon: faGears, content: variables },
	]}
/>

{#snippet status()}
	<h1>Status Tab</h1>
	<p>Woah</p>
{/snippet}

{#snippet compose()}
	<h1>Compose Tab</h1>
{/snippet}

{#snippet variables()}
	<h1>Variables Tab</h1>
{/snippet}
