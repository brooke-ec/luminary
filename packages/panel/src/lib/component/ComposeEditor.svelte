<script lang="ts">
	import { catppuccinMacchiato } from "@catppuccin/codemirror";
	import { EditorView, keymap } from "@codemirror/view";
	import type { Attachment } from "svelte/attachments";
	import { indentWithTab } from "@codemirror/commands";
	import { yaml } from "@codemirror/lang-yaml";
	import { basicSetup } from "codemirror";

	let { content = $bindable() }: { content: string } = $props();

	let focused = $state(false);

	const editor: Attachment<HTMLElement> = (parent) => {
		const view = new EditorView({
			extensions: [
				yaml(),
				basicSetup,
				keymap.of([indentWithTab]),
				catppuccinMacchiato,

				// Update content on every change
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						content = update.state.doc.toString();
					}
				}),
			],
			doc: content,
			parent,
		});

		return () => {
			view.destroy();
		};
	};
</script>

<div class="container">
	{#if focused}
		<div class="message">Esc</div>
	{/if}
	<div onfocusin={() => (focused = true)} onfocusout={() => (focused = false)} class="editor" {@attach editor}></div>
</div>

<style lang="scss">
	.container {
		position: relative;

		display: flex;
		align-items: end;
		justify-content: end;
	}

	.message {
		position: absolute;
		z-index: 10;

		padding: 5px 10px;
		margin: 10px;

		background-color: var(--crust);
		border: var(--subtext0);
		border-radius: 5px;
	}

	.editor {
		background-color: var(--base);
		border-radius: 10px;
		padding: 10px;
		width: 100%;

		& > :global(.cm-focused) {
			outline: none;
		}

		& :global(.cm-button) {
			border-color: var(--subtext0);
			background: transparent;
		}

		& :global(.cm-panels) {
			border-radius: 10px;
			padding: 5px;
			border: none;
		}

		& :global(.cm-textfield) {
			border-color: var(--subtext0);
		}
	}
</style>
