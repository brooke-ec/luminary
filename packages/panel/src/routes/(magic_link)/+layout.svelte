<script lang="ts">
	import LoaderButton from "$lib/component/LoaderButton.svelte";
	import { goto } from "$app/navigation";
	import { api } from "$lib";

	let { data, children } = $props();

	let loading = $state(false);
	let password = $state("");

	async function set_password(e: SubmitEvent) {
		if (!data.magic) return;

		e.preventDefault();
		loading = true;
		try {
			await api.client.POST("/api/auth/reset/{token}", {
				params: { path: { token: data.magic.token } },
				body: { password },
			});

			console.log({ username: data.magic.username, password });

			await api.login({ username: data.magic.username, password });
			await goto("/");
		} finally {
			loading = false;
		}
	}
</script>

<main class="full flex center">
	<div class="island flexc center gap-20">
		{@render children()}
		<form class="flexc gap-20" onsubmit={set_password}>
			<div>
				<label for="password">Password</label>
				<input required minlength="1" id="password" type="password" bind:value={password} />
			</div>
			<div>
				<label for="confirm-password">Confirm Password</label>
				<input required pattern={password} minlength="1" id="confirm-password" type="password" />
			</div>
			<LoaderButton {loading}>
				{#snippet children(loading)}
					{#if loading}
						Logging in...
					{:else}
						Log In
					{/if}
				{/snippet}
			</LoaderButton>
		</form>
	</div>
</main>
