<script lang="ts">
	import { faUsers } from "@fortawesome/free-solid-svg-icons";
	import Tabs from "$lib/component/Tabs.svelte";
	import UserList from "./UserList.svelte";
</script>

<svelte:head>
	<title>Server - Luminary</title>
</svelte:head>

<Tabs tabs={[{ icon: faUsers, label: "users", content: users }]} />

{#snippet users()}
	<UserList />
{/snippet}
