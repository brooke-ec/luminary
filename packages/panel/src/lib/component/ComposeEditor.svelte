<script lang="ts">
	import { catppuccinMacchiato } from "@catppuccin/codemirror";
	import type { Attachment } from "svelte/attachments";

	import { yamlSchema } from "codemirror-json-schema/yaml";
	import * as autocomplete from "@codemirror/autocomplete";
	import * as language from "@codemirror/language";
	import * as commands from "@codemirror/commands";
	import { yaml } from "@codemirror/lang-yaml";
	import * as estate from "@codemirror/state";
	import * as view from "@codemirror/view";
	import * as lint from "@codemirror/lint";

	import schema from "../schema.json";

	let { content = $bindable() }: { content: string } = $props();

	let focused = $state(false);

	const editor: Attachment<HTMLElement> = (parent) => {
		const editor = new view.EditorView({
			extensions: [
				catppuccinMacchiato,
				yamlSchema(schema),
				yaml(),

				view.lineNumbers(),
				view.highlightSpecialChars(),
				commands.history(),
				language.foldGutter(),
				view.drawSelection(),
				view.dropCursor(),
				estate.EditorState.allowMultipleSelections.of(true),
				language.indentOnInput(),
				language.syntaxHighlighting(language.defaultHighlightStyle, { fallback: true }),
				language.bracketMatching(),
				autocomplete.closeBrackets(),
				autocomplete.autocompletion(),
				view.rectangularSelection(),
				view.crosshairCursor(),
				view.keymap.of([
					commands.indentWithTab,
					...autocomplete.closeBracketsKeymap,
					...commands.defaultKeymap,
					...commands.historyKeymap,
					...language.foldKeymap,
					...autocomplete.completionKeymap,
					...lint.lintKeymap,
				]),

				// Update content on every change
				view.EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						content = update.state.doc.toString();
					}
				}),
			],
			doc: content,
			parent,
		});

		return () => {
			editor.destroy();
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

		& :global(.cm-tooltip) {
			background-color: var(--mantle);
			box-shadow: 0 -2px 10px #00000080;

			// padding: 0 10px;
			border-radius: 5px;
			overflow: hidden;

			border: 1px solid var(--subtext0);

			& :global(.cm6-json-schema-hover) {
				padding: 0 10px;
			}
		}

		& :global(.cm-tooltip .cm-tooltip-arrow:before),
		& :global(.cm-tooltip .cm-tooltip-arrow:after) {
			display: none;
		}
	}
</style>
