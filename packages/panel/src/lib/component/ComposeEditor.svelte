<script lang="ts">
	import { faAlignLeft, faArrowRightFromBracket } from "@fortawesome/free-solid-svg-icons";
	import type { Attachment } from "svelte/attachments";

	import { indentationMarkers } from "@replit/codemirror-indentation-markers";
	import { catppuccinMacchiato } from "@catppuccin/codemirror";
	import { yamlSchema } from "codemirror-json-schema/yaml";
	import * as autocomplete from "@codemirror/autocomplete";
	import * as language from "@codemirror/language";
	import prettierYaml from "prettier/plugins/yaml";
	import * as commands from "@codemirror/commands";
	import { yaml } from "@codemirror/lang-yaml";
	import * as estate from "@codemirror/state";
	import * as view from "@codemirror/view";
	import * as lint from "@codemirror/lint";
	import * as prettier from "prettier";

	import schema from "../schema.json";
	import { error } from "$lib";
	import Fa from "svelte-fa";

	let { content = $bindable() }: { content: string } = $props();

	let focused = $state(false);

	async function format(view: view.EditorView) {
		const doc = view.state.doc.toString();

		try {
			const { formatted, cursorOffset } = await prettier.formatWithCursor(doc, {
				cursorOffset: view.state.selection.main.head,
				plugins: [prettierYaml],
				parser: "yaml",
			});

			if (formatted === doc) return;

			view.dispatch({
				changes: { from: 0, to: doc.length, insert: formatted },
				selection: { anchor: cursorOffset },
			});
		} catch (e) {
			error("Failed to format YAML", String(e));
		}
	}

	const editor: Attachment<HTMLElement> = (parent) => {
		const editor = new view.EditorView({
			extensions: [
				indentationMarkers(),
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

					// Format keybind
					{
						key: "Alt-f",
						run(view) {
							format(view);
							return true;
						},
					},
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
		<div class="messages">
			<div class="message"><Fa icon={faArrowRightFromBracket} /> Esc</div>
			<div class="message"><Fa icon={faAlignLeft} /> Alt F</div>
		</div>
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

	.messages {
		pointer-events: none;
		display: flex;
		position: absolute;
		gap: 10px;

		margin: 10px;
	}

	.message {
		display: flex;
		align-items: center;
		gap: 10px;

		z-index: 10;

		padding: 5px 10px;

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
