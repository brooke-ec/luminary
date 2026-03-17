<!--
	@component
	
	A Svelte component for a button that can show a loading state.
-->

<script lang="ts">
	import type { Snippet } from "svelte";

	let {
		style = "button",
		loading = false,
		children,
		disabled,
		onclick,
	}: {
		style?: "button" | "a" | "outline";
		children: Snippet<[boolean]> | string;
		onclick?: () => Promise<void>;
		disabled?: boolean;
		loading?: boolean;
	} = $props();
</script>

<button class="full {style}" disabled={loading || disabled} {onclick}>
	{#if loading}
		<span class="loader"></span>
	{/if}

	<div>
		{#if typeof children === "string"}
			{children}
		{:else}
			{@render children(loading)}
		{/if}
	</div>
</button>

<style lang="scss">
	button {
		display: flex;
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
</style>
