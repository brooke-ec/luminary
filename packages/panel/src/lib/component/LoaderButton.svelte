<!--
	@component
	
	A Svelte component for a button that can show a loading state.
-->

<script lang="ts">
	import type { Snippet } from "svelte";

	let {
		onclick,
		children,
		loading = $bindable(),
	}: { onclick?: () => Promise<void>; children: Snippet<[boolean]>; loading: boolean } = $props();
</script>

<button class="full" {onclick} disabled={loading}>
	{#if loading}
		<span class="loader"></span>
	{/if}

	<div>{@render children(loading)}</div>
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
		border: 3px solid #fff;
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
