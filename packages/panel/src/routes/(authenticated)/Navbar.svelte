<!--
    @component
    
    A Svelte component containing both the desktop and mobile Navbar.
    
-->

<script lang="ts">
	import Crane from "$lib/component/Crane.svelte";
	import { onNavigate } from "$app/navigation";
	import { slide } from "svelte/transition";
	import type { Snippet } from "svelte";
	import { page } from "$app/state";
	import { isMobile } from "$lib";
	import Fa from "svelte-fa";
	import {
		faMagnifyingGlass,
		faChevronLeft,
		faLayerGroup,
		faXmark,
		faBars,
		faGear,
		faPlusCircle,
		faLock,
	} from "@fortawesome/free-solid-svg-icons";

	const EXPANDED_KEY = "luminary-navbar-expanded";

	const PAGES = [
		{ icon: faLayerGroup, label: "Projects", href: "/projects" },
		{ icon: faPlusCircle, label: "Create", href: "/create" },
		"search",
		{ icon: faLock, label: "Admin", href: "/admin" },
		{ icon: faGear, label: "Settings", href: "/settings" },
	] satisfies ({ icon: any; label: string; href: string } | "search")[];

	let { children }: { children: Snippet<[]> } = $props();

	let expanded = $state(localStorage.getItem(EXPANDED_KEY) !== "false");
	let open = $state(false);

	let navbarWidth = $state(0);

	let returnable = $derived(page.url.pathname.split("/").length > 3);

	function toggleExpanded() {
		expanded = !expanded;
		localStorage.setItem(EXPANDED_KEY, expanded.toString());
	}

	function toggleOpen() {
		open = !open;
	}

	onNavigate(() => {
		open = false;
	});
</script>

{#snippet links()}
	{#each PAGES as entry}
		{#if entry === "search"}
			<button class="a entry" style="margin-bottom: auto">
				<div class="icon">
					<Fa icon={faMagnifyingGlass} />
				</div>
				<div class="label">Search</div>
			</button>
		{:else}
			{@const { icon, label, href } = entry}

			<a class="entry" {href} class:current={page.url.pathname.startsWith(href)}>
				<div class="icon">
					<Fa {icon} />
				</div>
				<div class="label">{label}</div>
			</a>
		{/if}
	{/each}
{/snippet}

<div class="container">
	{#if isMobile()}
		<div style:min-height="48px"></div>

		<nav class:open>
			<div class="titlebar">
				<button class="a" onclick={toggleOpen}>
					<Fa icon={open ? faXmark : faBars} />
				</button>
				{#if returnable}
					<a href="../">
						<Fa icon={faChevronLeft} />
					</a>
				{:else}
					<h2 class="flex center gap-5" style="margin-left: 10px;"><Crane /> Luminary</h2>
				{/if}
			</div>
			{#if open}
				<div class="list" transition:slide>
					<div class="flexc expanded">{@render links()}</div>
				</div>
			{/if}
		</nav>
	{:else}
		<div style:min-width="{navbarWidth}px"></div>

		<nav class:expanded bind:clientWidth={navbarWidth}>
			{@render links()}

			<button class="a entry" onclick={toggleExpanded} aria-label="{expanded ? 'collapse' : 'expand'} sidebar">
				<div class="icon">
					<Fa icon={expanded ? faChevronLeft : faBars} />
				</div>
				<div class="label">Collapse</div>
			</button>
		</nav>
	{/if}

	<div class="flexc gap-20 full">
		{#if returnable && !isMobile()}
			<a href="../"><Fa icon={faChevronLeft} /> Back</a>
		{/if}
		<main class="full">
			{@render children()}
		</main>
	</div>
</div>

<style lang="scss">
	$navbar-width: 125px;

	.container {
		display: flex;

		@media (max-width: 425px) {
			flex-direction: column;
		}
	}

	nav {
		background-color: var(--crust);

		position: fixed;
		z-index: 100;
		left: 0;
		top: 0;

		display: flex;
		flex-direction: column;

		@media (max-width: 425px) {
			background-color: var(--crust);

			transition: height 250ms ease;
			height: 48px;
			width: 100%;

			&.open {
				height: 100dvh;
				width: 100%;
			}
		}

		@media (min-width: 426px) {
			transition: width 250ms ease;
			height: 100dvh;
			width: 48px;

			&.expanded {
				width: #{$navbar-width};
			}
		}
	}

	.titlebar {
		justify-content: space-between;
		flex-direction: row-reverse;
		align-items: center;
		display: flex;

		flex-basis: 48px;
		height: 50px;
		width: 100%;

		& > a,
		& > button {
			width: 48px;
			height: 100%;

			font-size: 20px;
			color: var(--subtext0);

			display: flex;
			justify-content: center;
			align-items: center;
		}
	}

	.list {
		transition: flex-basis 250ms ease;
		flex-basis: 0px;
		overflow: hidden;
	}

	.open .list {
		flex-basis: calc(100% - 48px);
	}

	.entry {
		justify-content: left;
		align-items: center;
		display: flex;

		padding: 0;

		height: 48px;
		border-radius: 0;

		color: var(--subtext0);

		&:hover,
		&.current {
			color: var(--mauve);
		}
	}

	.expanded .label {
		flex-basis: #{$navbar-width - 48px};
	}

	.icon {
		display: flex;
		justify-content: center;
		align-items: center;

		width: 48px;
		font-size: 23px;

		flex-basis: 48px;
	}

	.label {
		overflow: hidden;
		transition: flex-basis 250ms ease;
		flex-basis: 0px;

		white-space: nowrap;
		font-size: 14px;
	}
</style>
