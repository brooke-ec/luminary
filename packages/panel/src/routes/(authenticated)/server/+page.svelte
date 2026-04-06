<script lang="ts">
	import { faServer, faUsers } from "@fortawesome/free-solid-svg-icons";
	import Tabs from "$lib/component/Tabs.svelte";
	import UserList from "./UserList.svelte";
	import { page } from "$app/state";
	import Fa from "svelte-fa";
</script>

<svelte:head>
	<title>Server - Luminary</title>
</svelte:head>

<div class="flexc gap-10">
	<!-- Title Bar -->
	<h1 class="flexr gap-10 center fit">
		<Fa icon={faServer} size="lg" />
		<div style="display: inline-block;">
			<div style="font-size: 22px;">{page.url.host}</div>
			<div class="subtext">Luminary v{__LUMINARY_VERSION__}</div>
		</div>
	</h1>

	<Tabs tabs={[{ icon: faUsers, label: "users", content: users }]} />
</div>

{#snippet users()}
	<UserList />
{/snippet}
