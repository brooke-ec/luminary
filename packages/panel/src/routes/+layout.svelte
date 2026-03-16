<script lang="ts">
	import "@fontsource-variable/open-sans";
	import "@fontsource/dejavu-mono";
	import "$lib/style.scss";

	import Toaster from "./Toaster.svelte";
	import Dialog from "./Dialog.svelte";

	let { children } = $props();
</script>

{@render children()}

<Toaster />
<Dialog />
