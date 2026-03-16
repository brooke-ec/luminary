<script lang="ts">
	import { goto } from "$app/navigation";
	import { api } from "$lib";

	async function login() {
		await api.login({
			username: "admin",
			password: "password",
		});

		goto("/");
	}
</script>

<button onclick={login}>Login</button>
