<script lang="ts">
	import { faArrowRightFromBracket, faGears, faUser } from "@fortawesome/free-solid-svg-icons";
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import Tabs from "$lib/component/Tabs.svelte";
	import Fa from "svelte-fa";
	import { api } from "$lib";

	let { data } = $props();
</script>

<div class="flexc gap-10">
	<!-- Title Bar -->
	<h1 class="flexr gap-10 center fit">
		<Fa icon={faUser} size="lg" />
		<div style="display: inline-block;">
			<div style="font-size: 22px;">{data.user.username}</div>
			<PromiseButton style="a" fit onclick={api.logout}>
				<div class="a flexr gap-5" style="color: var(--red)">
					<Fa icon={faArrowRightFromBracket} />
					Log out
				</div>
			</PromiseButton>
		</div>
	</h1>

	<Tabs tabs={[{ icon: faGears, label: "application", content: appearance }]} />
</div>

{#snippet appearance()}
	<div class="flexc gap-10">COMING SOON</div>
{/snippet}
