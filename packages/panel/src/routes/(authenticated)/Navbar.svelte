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
	} from "@fortawesome/free-solid-svg-icons";

	const EXPANDED_KEY = "luminary-navbar-expanded";

	const PAGES = [
		{ icon: faLayerGroup, label: "Projects", href: "/projects" },
		{ icon: faGear, label: "Settings", href: "/settings" },
		{ icon: faCircleUser, label: "User", href: "/user" },
	] satisfies { icon: any; label: string; href: string }[];

	let expanded = $state(localStorage.getItem(EXPANDED_KEY) === "true");

	let clientWidth = $state(0);

	function toggleExpanded() {
		expanded = !expanded;
		localStorage.setItem(EXPANDED_KEY, expanded.toString());
	}
</script>

<div style:width="{clientWidth}px"></div>

<nav class:expanded bind:clientWidth>
	{#each PAGES as { icon, label, href }}
		<a {href} class:current={page.url.pathname.startsWith(href)}>
			<div class="icon">
				<Fa {icon} />
			</div>
			<div class="label">{label}</div>
		</a>
	{/each}

	<button class="a" style="margin-top: auto">
		<div class="icon">
			<Fa icon={faMagnifyingGlass} />
		</div>
		<div class="label">Search</div>
	</button>

	<button class="a" onclick={toggleExpanded} aria-label="{expanded ? 'collapse' : 'expand'} sidebar">
		<div class="icon">
			<Fa icon={expanded ? faChevronLeft : faBars} />
		</div>
		<div class="label">Collapse</div>
	</button>
</nav>

<style lang="scss">
	$navbar-width: 125px;

	nav {
		background-color: var(--crust);

		display: flex;
		flex-direction: column;
		gap: 10px;

		transition: width 250ms ease;
		height: 100dvh;
		width: 48px;

		position: fixed;
		left: 0;
		top: 0;

		& > * {
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

		&.expanded {
			width: #{$navbar-width};

			& > * > .label {
				flex-basis: #{$navbar-width - 48px};
			}
		}
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
