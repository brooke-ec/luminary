<script lang="ts">
	import Logo from "$lib/component/Crane.svelte";
	let { data } = $props();
</script>

<div class="flexc center">
	<h2 class="sub">
		<Logo />
		Luminary
	</h2>
	<h1>Reset Password</h1>
</div>
<p>
	Hi {data.magic!.username}! Please enter a new password to access your account.
</p>
