<script lang="ts">
	import { faBan, faKey, faPlus, faSignature, faUserCircle, faWrench } from "@fortawesome/free-solid-svg-icons";
	import LoaderButton from "$lib/component/LoaderButton.svelte";
	import CopyBox from "$lib/component/CopyBox.svelte";
	import { api, isMobile, openDialog } from "$lib";
	import Fa from "svelte-fa";

	type LuminaryUser = api.components["schemas"]["luminary.api.auth.LuminaryUser"];

	let users: LuminaryUser[] = $state([]);

	async function refresh() {
		users = await api.client.GET("/api/auth/users").then((res) => res.data!);
	}

	let username = $state("");
	let loading = $state(false);
	async function create_user() {
		loading = true;
		const response = await api.client.POST("/api/auth/users", {
			body: { username },
		});

		let url = new URL(`invite?token=${response.data!}`, window.location.toString()).toString();

		refresh();
		openDialog({
			content: success,
			title: `User ${username} created!`,
			parameters: { username, url },
		});
		loading = false;
	}

	async function delete_user(uuid: string) {
		await api.client.DELETE("/api/auth/users/{user}", { params: { path: { user: uuid } } });
		refresh();
	}

	refresh();
</script>

{#snippet create()}
	<p>This form will create a new Luminary user with the given username.</p>
	<p>The next page will show a reset password link to set up the account.</p>
	<form class="flexc gap-20" onsubmit={create_user}>
		<div>
			<label for="username">Username</label>
			<input required minlength="1" id="username" type="text" bind:value={username} />
		</div>
		<LoaderButton {loading} fit>
			{#snippet children(loading)}
				{#if loading}
					Creating user...
				{:else}
					Create user
				{/if}
			{/snippet}
		</LoaderButton>
	</form>
{/snippet}

{#snippet success({ username, url }: { username: string; url: string })}
	<p>Send {username} the link below so that they can set up their account:</p>
	<CopyBox value={url} />
{/snippet}

<table>
	<thead>
		<tr>
			<th></th>
			{#if !isMobile()}
				<th><Fa icon={faKey} /> uuid</th>
			{/if}
			<th><Fa icon={faSignature} /> username</th>
			<th class="actions-col"><Fa icon={faWrench} /> actions</th>
		</tr>
	</thead>
	<tbody>
		{#each users as user}
			<tr>
				<td><Fa icon={faUserCircle} /></td>
				{#if !isMobile()}
					<td class="subtext">{user.uuid}</td>
				{/if}
				<td>{user.username}</td>
				<td class="actions-col">
					<button class="outline" onclick={() => delete_user(user.uuid)}>
						<Fa icon={faBan} /> Delete
					</button>
				</td>
			</tr>
		{/each}
	</tbody>
</table>

<div class="flexr gap-10">
	<button
		class="outline flexr gap-5 center"
		onclick={() => {
			openDialog({ title: "Create User", content: create });
		}}
	>
		<Fa icon={faPlus} /> Create user
	</button>
</div>

<style lang="scss">
	table {
		border-collapse: collapse;
	}

	thead th {
		border-bottom: 1px solid var(--subtext0);
	}

	th,
	td {
		text-align: left;
		white-space: nowrap;
		padding: 5px 10px;
	}

	.actions-col {
		text-align: right;
		width: 100%;
	}
</style>
