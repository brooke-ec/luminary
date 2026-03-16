<script lang="ts">
	import Logo from "$lib/component/Crane.svelte";
	import { goto } from "$app/navigation";
	import { api } from "$lib";
	import PromiseButton from "$lib/component/PromiseButton.svelte";

	let credentials = $state({
		username: "",
		password: "",
	});

	async function login() {
		await api.login(credentials);
		await goto("/");
	}
</script>

<div class="full flex center">
	<div class="island flexc center gap-20">
		<div class="flexc center">
			<h2 class="sub">
				<Logo />
				Luminary
			</h2>
			<h1>Log In</h1>
		</div>
		<form class="flexc gap-20">
			<div>
				<label for="username">Username</label>
				<input required minlength="1" id="username" type="text" bind:value={credentials.username} />
			</div>
			<div>
				<label for="password">Password</label>
				<input required minlength="1" id="password" type="password" bind:value={credentials.password} />
			</div>
			<PromiseButton onclick={login}>
				{#snippet children(loading)}
					{#if loading}
						Logging in...
					{:else}
						Log In
					{/if}
				{/snippet}
			</PromiseButton>
		</form>
	</div>
</div>
