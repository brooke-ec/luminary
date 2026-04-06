<script lang="ts" module>
	import { faCirclePlus, faGear, faLayerGroup, faListUl, faServer } from "@fortawesome/free-solid-svg-icons";
	import type { IconDefinition } from "@fortawesome/fontawesome-common-types";
	import { getProjects } from "$lib/api";
	import { Dialog } from "melt/builders";

	// svelte-ignore non_reactive_update
	let dialog: Dialog;
	let input: HTMLInputElement;

	export function openSearch() {
		dialog.open = true;
		input.value = "";
		input.focus();
	}

	interface SearchOption {
		icon: IconDefinition;
		label: string;
		href: string;
		aliases?: string;
	}

	let options: Record<string, SearchOption> = $derived({
		projects: {
			icon: faListUl,
			label: "Projects",
			href: "/projects",
		},
		create: {
			icon: faCirclePlus,
			label: "Create Project",
			href: "/create",
			aliases: "new project",
		},
		server: {
			icon: faServer,
			label: "Server Settings",
			href: "/server",
		},
		settings: {
			icon: faGear,
			label: "User Settings",
			href: "/settings",
		},
		...Object.fromEntries(
			Object.values(getProjects()).map((project) => [
				`project ${project.name}`,
				{
					icon: faLayerGroup,
					label: project.name,
					href: `/projects/${project.name}`,
					aliases: `project ${project.name}`,
				},
			]),
		),
	});
</script>

<script lang="ts">
	import { Combobox } from "melt/builders";
	import { goto } from "$app/navigation";
	import Fa from "svelte-fa";

	dialog = new Dialog();
	const combobox = new Combobox<keyof typeof options>({
		onValueChange(value) {
			if (!value) return;

			combobox.value = undefined;
			goto(options[value].href);
			dialog.open = false;
		},
	});

	function normalise(str: string) {
		return str.trim().toLowerCase().replaceAll(/\s/g, "");
	}

	const filtered = $derived.by(() => {
		if (!combobox.touched) return Object.entries(options);

		const search = normalise(combobox.inputValue);
		return Object.entries(options).filter(
			([_, o]) =>
				normalise(o.label).includes(search) ||
				(o.aliases === undefined ? false : normalise(o.aliases).includes(search)),
		);
	});

	function onkeydown(event: KeyboardEvent) {
		if (event.key === "Escape") dialog.open = false;
		else combobox.input.onkeydown(event);
	}
</script>

<dialog {...dialog.content}>
	<h1><label {...combobox.label}>Navigate</label></h1>
	<input bind:this={input} type="text" {...combobox.input} autocomplete="off" {onkeydown} />

	<div class="options" {...combobox.content} popover={undefined} inert={undefined}>
		{#each filtered as [key, option] (option)}
			<div class="option" {...combobox.getOption(key)}>
				<Fa icon={option.icon} />
				{option.label}
			</div>
		{:else}
			<span>No results found</span>
		{/each}
	</div>
</dialog>

<style lang="scss">
	dialog {
		background: none;
		border: none;

		color: inherit;
	}

	.options {
		// Override popover styles
		transform-origin: unset !important;
		min-width: unset !important;
		position: unset !important;
		width: unset !important;
		left: unset !important;
		top: unset !important;

		margin-top: 10px;

		overflow-y: auto;

		display: flex;
		flex-direction: column;
		gap: 5px;
	}

	.option {
		background: var(--surface0);
		padding: 5px;

		border-radius: 10px;
		border: 2px solid transparent;

		font-size: 18px;
		cursor: pointer;

		display: flex;
		align-items: center;
		gap: 5px;

		&[data-highlighted] {
			border-color: var(--flamingo);
		}
	}

	dialog::backdrop {
		background: rgba(0, 0, 0, 0.2);
		backdrop-filter: blur(2px);
	}

	dialog[data-open] {
		pointer-events: all;
		opacity: 1;
		scale: 1;
	}

	[data-melt-dialog-overlay][data-open] {
		opacity: 1;
	}
</style>
