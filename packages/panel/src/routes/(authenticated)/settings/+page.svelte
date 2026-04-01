<script lang="ts">
	import { faArrowRightFromBracket, faGears, faUser } from "@fortawesome/free-solid-svg-icons";
	import PromiseButton from "$lib/component/PromiseButton.svelte";
	import Tabs from "$lib/component/Tabs.svelte";
	import Fa from "svelte-fa";
	import { api } from "$lib";

	const THEME_KEY = "luminary-theme";
	const FORMAT_ON_SAVE_KEY = "luminary-format-on-save";

	let { data } = $props();

	let theme = $state(localStorage.getItem(THEME_KEY) ?? "macchiato");

	$effect(() => {
		localStorage.setItem(THEME_KEY, theme);
		document.documentElement.setAttribute("class", theme);
	});

	let formatOnSave = $state(localStorage.getItem(FORMAT_ON_SAVE_KEY) == "true");

	$effect(() => {
		localStorage.setItem(FORMAT_ON_SAVE_KEY, String(formatOnSave));
	});
</script>

<svelte:head>
	<title>Settings - Luminary</title>
</svelte:head>

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
	<div class="flexc gap-10">
		<div class="fit">
			<h2>Editor</h2>
			<div class="fit">
				<input type="checkbox" id="format-on-save" bind:checked={formatOnSave} />
				<label for="format-on-save">Format on save</label>
			</div>
		</div>
		<div class="fit">
			<h2>Theme</h2>
			<div>
				<input type="radio" id="theme-macchiato" name="theme" value="macchiato" bind:group={theme} />
				<label for="theme-macchiato">🌺 Macchiato</label>
			</div>
			<div>
				<input type="radio" id="theme-frappe" name="theme" value="frappe" bind:group={theme} />
				<label for="theme-frappe">🪴 Frappé</label>
			</div>
			<div>
				<input type="radio" id="theme-mocha" name="theme" value="mocha" bind:group={theme} />
				<label for="theme-mocha">🌿 Mocha</label>
			</div>
			<div>
				<input type="radio" id="theme-latte" name="theme" value="latte" bind:group={theme} />
				<label for="theme-latte">🌻 Latte</label>
			</div>
		</div>
	</div>
{/snippet}
