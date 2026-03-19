<script lang="ts">
	import { faUsers } from "@fortawesome/free-solid-svg-icons";
	import Tabs from "$lib/component/Tabs.svelte";
	import UserList from "./UserList.svelte";
</script>

<Tabs tabs={[{ icon: faUsers, label: "users", content: users }]} />

{#snippet users()}
	<UserList />
{/snippet}
