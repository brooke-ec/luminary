<script lang="ts">
	import type { paths } from "$lib/openapi";
	import createClient from "openapi-fetch";

	const client = createClient<paths>({ baseUrl: "./" });
	const login = client.POST("/api/auth/login", {
		body: {
			username: "admin",
			password: "password",
		},
	});
</script>

{#await login}
	Loading...
{:then response}
	{response.data?.token}
{/await}
