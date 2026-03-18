<!--
	@component

	A Svelte component for displaying toast notifications.
	This should be mounted in a top-level `layout.svelte` so that toasts are visible on every page.

	Toasts can be displayed by calling the `addToast` function exported from this component.
-->

<script lang="ts" module>
	import type { IconDefinition } from "@fortawesome/fontawesome-common-types";

	/**
	 * The interface representing the content of a given toast notification.
	 */
	export interface ToastData {
		color: string;
		icon: IconDefinition;
		title: string;
		details?: string | string[];
	}

	const toaster = new Toaster<ToastData>({ closeDelay: 15 * 1000 });

	/**
	 * Adds a new toast notification to the toaster.
	 * @param props The options used to create the toast.
	 */
	export const addToast = toaster.addToast;
</script>

<script lang="ts">
	import { faXmark } from "@fortawesome/free-solid-svg-icons";
	import { openDialog } from "./Dialog.svelte";
	import { Progress } from "melt/builders";
	import { Toaster } from "melt/builders";
	import { fly } from "svelte/transition";
	import { flip } from "svelte/animate";
	import { Fa } from "svelte-fa";
</script>

{#snippet content(data: ToastData)}
	{#if Array.isArray(data.details)}
		<ul class="details">
			{#each data.details as detail}
				<li>{detail}</li>
			{/each}
		</ul>
	{:else if typeof data.details === "string"}
		<p class="details">{data.details}</p>
	{:else}
		<p class="details">No additional information provided.</p>
	{/if}
{/snippet}

{#snippet title(data: ToastData)}
	<span style:color="var(--{data.color})" style="padding-right: 5px;">
		<Fa icon={data.icon} />
	</span>
	<span>{data.title}</span>
{/snippet}

<div {...toaster.root} class="root">
	{#each toaster.toasts as toast (toast.id)}
		{@const progress = new Progress({ value: () => toast.percentage })}

		<div
			class="toast"
			{...toast.content}
			animate:flip={{ duration: 250 }}
			transition:fly={{ y: 20, duration: 250 }}
		>
			<div class="row">
				<h3 {...toast.title}>
					<span style:color="var(--{toast.data.color})" style="padding-right: 1px;">
						<Fa icon={toast.data.icon} translateY="0.1" />
					</span>
					<span>{toast.data.title}</span>
				</h3>
				<button class="a" style:color="inherit" {...toast.close} aria-label="dismiss alert">
					<Fa icon={faXmark} />
				</button>
			</div>
			<div class="row">
				{#if toast.data.details}
					<button
						class="a"
						{...toast.description}
						onclick={() => openDialog({ title, content, parameters: toast.data })}
					>
						More Information
					</button>
				{/if}
				<div {...progress.root}>
					<div {...progress.progress} style:background="var(--{toast.data.color})"></div>
				</div>
			</div>
		</div>
	{/each}
</div>

<style lang="scss">
	.root {
		font-family: inherit;
		color: inherit;

		position: fixed;
		inset: auto;
		bottom: 0px;
		right: 0px;

		margin: 0px;
		padding: 20px;

		background: none;
		border: none;

		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 0.5rem;

		width: 400px;
		max-width: calc(100vw - 40px);
		height: 100vh;
		overflow: hidden;
		pointer-events: none;

		@media (max-width: 425px) {
			top: 0;
			right: 0;
			bottom: 0;
			left: 0;
			box-sizing: border-box;
			min-width: 100dvw;
			min-height: 100dvh;
			padding: 10px;
			margin: 0;
		}
	}

	.toast {
		background-color: var(--surface0);
		box-shadow: 0 -2px 10px #00000080;

		pointer-events: all;
		overflow: hidden;

		border-radius: 10px;
		font-size: 14px;
		padding: 10px;
		box-sizing: border-box;

		display: flex;
		flex-direction: column;
		width: 100%;
		gap: 5px;
	}

	.row {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	h3 {
		font-size: inherit;
		font-weight: bold;

		display: flex;
		align-items: center;
		gap: 5px;
	}

	[data-melt-progress-root] {
		background-color: var(--base);

		overflow: hidden;

		border-radius: 4px;
		height: 4px;
		width: 60px;
	}

	[data-melt-progress-progress] {
		transform: translateX(calc((100% - var(--progress)) * -1));
		height: 100%;
	}

	.details {
		font-family: "DejaVu Mono", monospace;
		background-color: var(--crust);
		flex-direction: column;
		white-space: pre-line;
		word-wrap: break-word;
		border-radius: 5px;
		margin-top: 15px;
		font-size: 14px;
		display: flex;
		padding: 10px;
		gap: 10px;
	}

	ul.details {
		padding: 10px 10px 10px 30px;
	}
</style>
