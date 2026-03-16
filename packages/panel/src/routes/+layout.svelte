<script lang="ts">
	import "@fontsource-variable/open-sans";
	import "@fontsource/dejavu-mono";
	import "$lib/style.scss";

	import Toaster from "./Toaster.svelte";
	import Dialog from "./Dialog.svelte";

	let { children } = $props();
</script>

<Toaster />
<Dialog />

<main>
	{@render children()}
</main>

<style lang="scss">
	main {
		height: calc(100% - 20px);
		max-width: 1000px;
		margin: 0 auto;
		padding: 10px;
	}
</style>
