<!--
    @component
    
    A Svelte component providing tab functionality through the given list of snippets.

    On smaller screens, this transforms into an accordian component.
    
-->

<script lang="ts">
	import type { IconDefinition } from "@fortawesome/fontawesome-common-types";
	import { Accordion } from "melt/builders";
	import { fade, slide } from "svelte/transition";
	import type { Snippet } from "svelte";
	import Fa from "svelte-fa";
	import { faChevronDown, faChevronUp } from "@fortawesome/free-solid-svg-icons";

	let { tabs }: { tabs: { label: string; icon: IconDefinition; content: Snippet<[]> }[] } = $props();

	const accordion = new Accordion();

	let windowWidth = $state(0);
	let mobile = $derived(windowWidth <= 425);

	$effect(() => {
		if (accordion.value === undefined && !mobile) {
			accordion.value = tabs[0].label;
		}
	});
</script>

<svelte:window bind:innerWidth={windowWidth} />

<div {...accordion.root}>
	<div
		class:container={!mobile}
		class:flexc={mobile}
		class="flexr gap-10"
		style:width={mobile ? "100%" : "fit-content"}
	>
		{#each tabs as tab (tab.label)}
			{@const item = accordion.getItem({ id: tab.label })}
			<div class:container={mobile}>
				<button
					class="a switch"
					{...item.trigger}
					class:active={item.isExpanded}
					aria-label="switch to {tab.label} tab"
				>
					<Fa icon={tab.icon} translateY="0.1" />
					<span {...item.heading}>{tab.label}</span>

					{#if mobile}
						<Fa icon={item.isExpanded ? faChevronDown : faChevronUp} style="margin-left: auto;" />
					{/if}
				</button>
				{#if mobile}
					{#if item.isExpanded}
						<div style="overflow: hidden;" transition:slide={{ duration: 250 }}>
							<hr style="margin-top: 15px;" />
							<div {...item.content}>
								{@render tab.content()}
							</div>
						</div>
					{/if}
				{/if}
			</div>
		{/each}
	</div>

	{#if !mobile}
		<div class="container">
			{#each tabs as tab (tab.label)}
				{@const item = accordion.getItem({ id: tab.label })}
				{#if item.isExpanded}
					<div {...item.content} in:fade={{ duration: 125 }}>
						{@render tab.content()}
					</div>
				{/if}
			{/each}
		</div>
	{/if}
</div>

<style lang="scss">
	.container {
		background-color: var(--surface0);
		border-radius: 10px;

		padding: 10px;

		@media (min-width: 426px) {
			margin-bottom: 10px;
		}
	}

	.switch {
		display: flex;
		align-items: center;
		gap: 8px;

		text-transform: lowercase;
		color: var(--text);

		width: 100%;

		@media (max-width: 425px) {
			font-size: 18px;
		}

		&:hover {
			text-decoration: none;
			color: var(--mauve);
		}

		&.active {
			color: var(--mauve);

			@media (min-width: 426px) {
				pointer-events: none;
			}
		}
	}
</style>
