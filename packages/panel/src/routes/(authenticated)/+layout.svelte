<script lang="ts">
	import Navbar from "./Navbar.svelte";

	let { children } = $props();
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
