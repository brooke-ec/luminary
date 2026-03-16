<script lang="ts" module>
	import type { IconDefinition } from "@fortawesome/fontawesome-common-types";

	type ToastData = {
		color: string;
		icon: IconDefinition;
		title: string;
		description: string;
	};

	const toaster = new Toaster<ToastData>({ closeDelay: 15 * 1000 });
	export const addToast = toaster.addToast;
</script>

<script lang="ts">
	import { faXmark, faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
	import { faCircleXmark } from "@fortawesome/free-regular-svg-icons";
	import { Progress } from "melt/builders";
	import { Toaster } from "melt/builders";
	import { fly } from "svelte/transition";
	import { flip } from "svelte/animate";
	import { Fa } from "svelte-fa";
</script>

<button
	on:click={() =>
		addToast({
			data: {
				color: "peach",
				icon: faTriangleExclamation,
				title: "Something could go wrong",
				description: "More Information",
			},
		})}
>
	Add Warning
</button>

<button
	on:click={() =>
		addToast({
			data: { color: "red", icon: faCircleXmark, title: "Something went wrong", description: "More Information" },
		})}
>
	Add Error
</button>

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
				<div {...toast.description}>{toast.data.description}</div>
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
		height: 100vh;
		overflow: hidden;
		pointer-events: none;
	}

	.toast {
		background-color: var(--surface0);
		box-shadow: 0 -2px 10px #00000080;

		pointer-events: all;
		overflow: hidden;

		border-radius: 10px;
		font-size: 14px;
		padding: 10px;

		display: flex;
		flex-direction: column;
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
</style>
