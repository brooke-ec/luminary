<script lang="ts">
	import Navbar from "./Navbar.svelte";
	import { api } from "$lib";

	let { children } = $props();

	$effect(() => api.subscribe(fetch));
</script>

<div class="container">
	<Navbar />

	<div class="full">
		{@render children()}
	</div>
</div>

<style lang="scss">
	.container {
		display: flex;

		@media (max-width: 425px) {
			flex-direction: column;
		}
	}
</style>
