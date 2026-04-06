<script lang="ts">
	import Navbar from "./Navbar.svelte";
	import { api } from "$lib";
	import Search from "./Search.svelte";

	let { children } = $props();

	$effect(() => api.subscribe(fetch));
</script>

<Navbar {children} />
<Search />
