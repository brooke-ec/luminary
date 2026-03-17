<script lang="ts">
	import { Tooltip } from "melt/builders";
	import type { Snippet } from "svelte";

	type Placement = NonNullable<NonNullable<NonNullable<Tooltip["floatingConfig"]>["computePosition"]>["placement"]>;

	let {
		children,
		content,
		placement = "top",
	}: { content: string; children: Snippet<[]>; placement?: Placement } = $props();

	const tooltip = new Tooltip({
		openDelay: 0,
		disableHoverableContent: true,
		// svelte-ignore state_referenced_locally
		floatingConfig: { computePosition: { placement } },
	});
</script>

<div class="trigger" {...tooltip.trigger}>
	{@render children()}
</div>

<div {...tooltip.content} class="tooltip">
	<div class="arrow" {...tooltip.arrow}></div>
	<span>{content}</span>
</div>

<style lang="scss">
	.trigger {
		width: fit-content;
		cursor: help;
	}

	.tooltip {
		position: relative;
		background-color: var(--overlay0);
		box-shadow: 0 -2px 10px #00000080;
		color: inherit;

		border-radius: 5px;
		border: none;

		padding: 5px;
		margin: 0;
	}

	.arrow {
		position: absolute;
		width: 8px;
		height: 8px;
		background-color: var(--overlay2);
		transform: rotate(45deg);
	}

	.tooltip[data-side="top"] .arrow {
		bottom: -4px;
	}

	.tooltip[data-side="bottom"] .arrow {
		top: -4px;
	}

	.tooltip[data-side="left"] .arrow {
		right: -4px;
	}

	.tooltip[data-side="right"] .arrow {
		left: -4px;
	}
</style>
