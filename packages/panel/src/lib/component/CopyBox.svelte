<script lang="ts">
	import { faCheck, faCopy } from "@fortawesome/free-solid-svg-icons";
	import type { Attachment } from "svelte/attachments";
	import Tooltip from "./Tooltip.svelte";
	import { sleep } from "$lib";
	import Fa from "svelte-fa";

	let {
		value,
	}: {
		value: string;
	} = $props();

	let copied = $state(false);

	async function copy() {
		if (copied) return;

		await navigator.clipboard.writeText(value);
		copied = true;

		await sleep(1000);

		copied = false;
	}

	const select: Attachment<HTMLInputElement> = (el) => {
		const select = () => el.select();
		el.addEventListener("click", select);
		el.addEventListener("focusin", select);

		() => {
			el.removeEventListener("click", select);
			el.removeEventListener("focusin", select);
		};
	};
</script>

<div class="container">
	<input type="text" readonly {value} aria-label="Copy text" {@attach select} />
	<Tooltip content={copied ? "Copied!" : "Copy to clipboard"}>
		<button type="button" onclick={copy}>
			<Fa icon={copied ? faCheck : faCopy} />
		</button>
	</Tooltip>
</div>

<style lang="scss">
	.container {
		display: flex;
		gap: 5px;

		width: fit-content;

		background-color: var(--surface1);
		border-radius: 10px;

		display: flex;
		align-items: center;

		input {
			outline: none !important;
		}

		button {
			background: none;
			color: inherit;
		}
	}
</style>
