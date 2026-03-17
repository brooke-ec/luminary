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
	import type { ComponentProps, Snippet } from "svelte";
	import LoaderButton from "./LoaderButton.svelte";

	let {
		"aria-label": ariaLabel,
		onclick,
		children,
		disabled,
		loading,
		style,
	}: {
		style?: ComponentProps<typeof LoaderButton>["style"];
		children: Snippet<[boolean]> | string;
		onclick: () => Promise<any>;
		"aria-label"?: string;
		disabled?: boolean;
		loading?: boolean;
	} = $props();

	let waiting = $state(false);

	async function handleClick() {
		waiting = true;
		try {
			await onclick();
		} finally {
			waiting = false;
		}
	}
</script>

<LoaderButton onclick={handleClick} {disabled} {style} {children} loading={waiting || loading} aria-label={ariaLabel} />
