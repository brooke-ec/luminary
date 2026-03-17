<script lang="ts" module>
	let windowWidth = $state(0);
	let mobile = $derived(windowWidth <= 425);

	/**
	 * @returns true if the screen is small enough to be considered a mobile device.
	 */
	export const isMobile = () => mobile;
</script>

<script lang="ts">
	import "@fontsource-variable/open-sans";
	import "@fontsource/dejavu-mono";
	import "$lib/style.scss";

	import Toaster from "./Toaster.svelte";
	import Dialog from "./Dialog.svelte";
	import { AnimationFrames } from "runed";

	let { children } = $props();

	new AnimationFrames(() => {
		windowWidth = window.innerWidth;
	});
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
