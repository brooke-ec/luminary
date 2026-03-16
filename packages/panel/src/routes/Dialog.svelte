<!--
	@component
	
	A Svelte component for displaying Dialogs/Modals/Popups.
	This should be mounted in a top-level `layout.svelte` so that toasts are visible on every page.

	Dialogs can be displayed by calling the `openDialog` function exported from this component.
	
-->

<script lang="ts" module>
	import type { Snippet } from "svelte";

	/**
	 * The interface representing the content of a given dialog.
	 */
	export interface DialogOptions<T> {
		title: string | Snippet<[T]>;
		content: Snippet<[T]>;
		parameters?: T;
	}

	let current: DialogOptions<any> | null = $state(null);

	/**
	 * Opens a new dialog with the given options.
	 * @param props The options used to create the dialog.
	 */
	export function openDialog<T>(props: DialogOptions<T>) {
		current = props;
	}

	/**
	 * Closes the currently open dialog, if there is one.
	 */
	export function closeDialog() {
		current = null;
	}
</script>

<script lang="ts">
	import { faXmark } from "@fortawesome/free-solid-svg-icons";
	import { fade } from "svelte/transition";
	import { Dialog } from "melt/builders";
	import Fa from "svelte-fa";

	const dialog = new Dialog({
		onOpenChange(value) {
			if (!value) closeDialog();
		},
	});

	$effect(() => {
		dialog.open = current !== null;
	});
</script>

<div {...dialog.overlay}></div>

<dialog {...dialog.content}>
	{#if current}
		{#key current}
			<div in:fade={{ duration: 250 }}>
				<div class="titlebar">
					<h1>
						{#if typeof current.title == "string"}
							{current.title}
						{:else}
							{@render current.title(current.parameters)}
						{/if}
					</h1>
					<button class="a" style:color="inherit" onclick={closeDialog}>
						<Fa icon={faXmark} scale={1.5} />
					</button>
				</div>
				<div class="content">
					{@render current.content(current.parameters)}
				</div>
			</div>
		{/key}
	{/if}
</dialog>

<style lang="scss">
	dialog {
		pointer-events: none;
		color: inherit;
		border: none;
		opacity: 0;

		box-shadow: 0 0 20px #00000033;
		background: var(--base);
		border-radius: 10px;

		display: flex;
		flex-direction: column;

		min-width: 75vw;
		min-height: 25vh;

		transition: ease 300ms;
		scale: 0.95;
	}

	dialog::backdrop {
		display: none;
	}

	dialog[data-open] {
		pointer-events: all;
		opacity: 1;
		scale: 1;
	}

	[data-melt-dialog-overlay] {
		backdrop-filter: blur(2px);
		position: fixed;
		width: 100%;
		height: 100%;
		background: rgba(0, 0, 0, 0.2);
		opacity: 0;
		transition: ease 300ms;
	}

	[data-melt-dialog-overlay][data-open] {
		opacity: 1;
	}

	.titlebar {
		justify-content: space-between;
		align-items: center;
		display: flex;
		gap: 20px;

		h1 {
			font-size: 25px;
		}

		button {
			font-size: 20px;
			padding: 0 10px;
		}
	}

	.content {
		flex-grow: 1;
	}
</style>
