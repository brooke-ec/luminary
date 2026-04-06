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
	import * as estate from "@codemirror/state";
	import * as view from "@codemirror/view";
	import * as lint from "@codemirror/lint";
	import * as prettier from "prettier";

	import schema from "../schema.json";
	import { error } from "$lib";
	import Fa from "svelte-fa";

	let { content = $bindable(), format: f = $bindable() }: { content: string; format?: () => Promise<void> } =
		$props();

	let initial = $state.snapshot(content);
	let focused = $state(false);

	const enterHandler = (view: view.EditorView) => {
		const { state } = view;
		const { from } = state.selection.main;
		const line = state.doc.lineAt(from);

		const change = line.text.substring(0, from - line.from).trim() !== "";

		let result = "\n";

		// Get the current indentation
		const currentIndent = line.text.match(/^(\s*)/)?.[0] ?? "";
		const unit = state.facet(language.indentUnit);

		// Check if line ends with a colon
		if (change && /:\s*(?:#.*)?$/.test(line.text)) {
			result += currentIndent + unit; // Increase indent for new line
		}

		// Check if line starts starts with a dash
		else if (change && /^\s*-/.test(line.text)) {
			result += currentIndent + "- "; // Inset a dash on new line
		}

		// Otherwise, maintain previous indentation
		else {
			result = "\n" + currentIndent;
		}

		// Update the document
		view.dispatch({
			changes: { from, to: state.selection.main.to, insert: result },
			selection: { anchor: from + result.length },
			scrollIntoView: true,
		});

		return true;
	};

	async function format(view: view.EditorView) {
		const doc = view.state.doc.toString();

		try {
			const { formatted, cursorOffset } = await prettier.formatWithCursor(doc, {
				cursorOffset: view.state.selection.main.head,
				plugins: [prettierYaml],
				parser: "yaml",
				useTabs: false,
			});

			// If document hasn't changed, return early
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
			doc: initial,
			parent,
			extensions: [
				language.indentUnit.of("  "),
				indentationMarkers(),
				catppuccinMacchiato,
				yamlSchema(schema),

				view.lineNumbers(),
				view.highlightSpecialChars(),
				commands.history(),
				language.foldGutter(),
				view.drawSelection(),
				view.dropCursor(),
				estate.EditorState.allowMultipleSelections.of(true),
				language.syntaxHighlighting(language.defaultHighlightStyle, { fallback: true }),
				language.bracketMatching(),
				autocomplete.closeBrackets(),
				autocomplete.autocompletion(),
				view.rectangularSelection(),
				view.crosshairCursor(),
				view.keymap.of([
					{ key: "Enter", run: enterHandler },
					{ key: "Ctrl-Enter", run: () => true },
					commands.indentWithTab,
					...autocomplete.closeBracketsKeymap,
					...commands.defaultKeymap,
					...commands.historyKeymap,
					...language.foldKeymap,
					...autocomplete.completionKeymap,
					...lint.lintKeymap,
					{ key: "Ctrl-Shift-z", run: commands.redo },

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
		});

		$effect(() => {
			if (content !== editor.state.doc.toString())
				editor.dispatch({
					changes: { from: 0, to: editor.state.doc.length, insert: content },
				});
		});

		f = () => format(editor);

		return () => {
			editor.destroy();
		};
	};
</script>

<div class="container">
	{#if focused}
		<div class="keybinds">
			<div class="keybind"><Fa icon={faArrowRightFromBracket} /> Esc</div>
			<div class="keybind"><Fa icon={faAlignLeft} /> Alt + F</div>
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

	.keybinds {
		z-index: 10;

		pointer-events: none;
		display: flex;
		position: absolute;
		gap: 10px;

		margin: 10px;
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
