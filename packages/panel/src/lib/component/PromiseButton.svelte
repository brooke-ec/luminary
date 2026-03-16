<!--
	@component
	
	A Svelte component for a button that runs a promise.

	The button will be disabled while the promise is running, and will show a loading spinner.

    The content of the button is determined by the children snippet. See the example below for usage:

    # Example

    ```svelte
    <PromiseButton onclick={login}>
			{#snippet children(loading)}
				{#if loading}
					Logging in...
				{:else}
					Log In
				{/if}
			{/snippet}
		</PromiseButton>
    ```	
-->

<script lang="ts">
	import type { Snippet } from "svelte";
	import LoaderButton from "./LoaderButton.svelte";

	let { onclick, children }: { onclick: () => Promise<void>; children: Snippet<[boolean]> } = $props();

	let loading = $state(false);

	async function handleClick() {
		loading = true;
		try {
			await onclick();
		} finally {
			loading = false;
		}
	}
</script>

<LoaderButton onclick={handleClick} bind:loading {children} />
