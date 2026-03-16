<script lang="ts">
	import "@fontsource-variable/open-sans";
	import "$lib/style.scss";

	import Toaster from "./Toaster.svelte";

	let { children } = $props();
</script>

{@render children()}

<Toaster />
