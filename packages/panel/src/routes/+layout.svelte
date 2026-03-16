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

<div class="main">
	{@render children()}
</div>

<style lang="scss">
	.main {
		height: calc(100% - 20px);
		padding: 10px;
	}
</style>
