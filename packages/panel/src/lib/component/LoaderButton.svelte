<!--
	@component
	
	A Svelte component for a button that can show a loading state.
-->

<script lang="ts">
	import type { Snippet } from "svelte";

	let {
		"aria-label": ariaLabel,
		style = "button",
		loading = false,
		fit = false,
		children,
		disabled,
		onclick,
	}: {
		style?: "button" | "a" | "outline" | "danger";
		children: Snippet<[boolean]> | string;
		"aria-label"?: string;
		onclick?: () => any;
		disabled?: boolean;
		loading?: boolean;
		fit?: boolean;
	} = $props();
</script>

<button class="{fit ? 'fit' : 'full'} {style}" disabled={loading || disabled} {onclick} aria-label={ariaLabel}>
	{#if loading}
		<span class="loader"></span>
	{/if}

	<div class="content">
		{#if typeof children === "string"}
			{children}
		{:else}
			{@render children(loading)}
		{/if}
	</div>
</button>

<style lang="scss">
	button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 10px;
	}

	.loader {
		width: 14px;
		height: 14px;
		border: 3px solid var(--text);
		border-bottom-color: transparent;
		border-radius: 50%;
		display: inline-block;
		box-sizing: border-box;
		animation: rotation 1s linear infinite;
	}

	@keyframes rotation {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	.content:empty {
		display: none;
	}
</style>
