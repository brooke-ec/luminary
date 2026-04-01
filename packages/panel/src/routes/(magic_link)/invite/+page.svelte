<script lang="ts">
	import Logo from "$lib/component/Crane.svelte";
	let { data } = $props();
</script>

<svelte:head>
	<title>Welcome - Luminary</title>
</svelte:head>

<div class="flexc center">
	<h2 class="sub">
		<Logo />
		Luminary
	</h2>
	<h1>Create Account</h1>
</div>
<h3>
	Welcome to Luminary {data.magic!.username}! 🎉
</h3>
<p>To get started, please set a password for your account.</p>
