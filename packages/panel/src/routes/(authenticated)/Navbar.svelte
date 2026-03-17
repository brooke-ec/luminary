<!--
    @component
    
    A Svelte component containing both the desktop and mobile Navbar.
    
-->

<script lang="ts">
	import { page } from "$app/state";
	import Fa from "svelte-fa";
	import {
		faBars,
		faChevronLeft,
		faCircleUser,
		faGear,
		faLayerGroup,
		faMagnifyingGlass,
		faXmark,
	} from "@fortawesome/free-solid-svg-icons";
	import { slide } from "svelte/transition";
	import { onNavigate } from "$app/navigation";

	const EXPANDED_KEY = "luminary-navbar-expanded";

	const PAGES = [
		{ icon: faLayerGroup, label: "Projects", href: "/projects" },
		{ icon: faGear, label: "Settings", href: "/settings" },
		{ icon: faCircleUser, label: "User", href: "/user" },
	] satisfies { icon: any; label: string; href: string }[];

	let expanded = $state(localStorage.getItem(EXPANDED_KEY) === "true");
	let open = $state(false);

	let navbarWidth = $state(0);
	let windowWidth = $state(0);

	let mobile = $derived(windowWidth <= 425);

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

<svelte:window bind:innerWidth={windowWidth} />

{#snippet links()}
	{#each PAGES as { icon, label, href }}
		<a class="entry" {href} class:current={page.url.pathname.startsWith(href)}>
			<div class="icon">
				<Fa {icon} />
			</div>
			<div class="label">{label}</div>
		</a>
	{/each}
{/snippet}

{#if mobile}
	<div style:min-height="48px"></div>

	<nav class:open>
		<div class="titlebar">
			<button class="a" onclick={toggleOpen}>
				<Fa icon={open ? faXmark : faBars} />
			</button>
			<a href="./">
				<Fa icon={faChevronLeft} />
			</a>
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

		<button class="a entry" style="margin-top: auto">
			<div class="icon">
				<Fa icon={faMagnifyingGlass} />
			</div>
			<div class="label">Search</div>
		</button>

		<button class="a entry" onclick={toggleExpanded} aria-label="{expanded ? 'collapse' : 'expand'} sidebar">
			<div class="icon">
				<Fa icon={expanded ? faChevronLeft : faBars} />
			</div>
			<div class="label">Collapse</div>
		</button>
	</nav>
{/if}

<style lang="scss">
	$navbar-width: 125px;

	nav {
		background-color: var(--crust);

		position: fixed;
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

		& > * {
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

		&.current {
			pointer-events: none;
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
		font-size: 30px;

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
