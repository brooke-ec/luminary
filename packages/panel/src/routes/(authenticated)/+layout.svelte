<script lang="ts">
	import Navbar from "./Navbar.svelte";
	import { api } from "$lib";

	let { children } = $props();

	$effect(() => api.subscribe(fetch));
</script>

<Navbar {children} />
