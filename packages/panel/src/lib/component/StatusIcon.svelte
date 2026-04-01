<script lang="ts" module>
	export type LuminaryStatus = components["schemas"]["luminary.core.model.LuminaryStatus"];
</script>

<script lang="ts">
	import { faCircleCheck, faCirclePause, faCircleXmark, faHeart } from "@fortawesome/free-regular-svg-icons";
	import { faHourglassStart, faPowerOff } from "@fortawesome/free-solid-svg-icons";
	import type { IconDefinition } from "@fortawesome/fontawesome-common-types";
	import type { components } from "$lib/api/openapi";
	import Fa from "svelte-fa";

	const STATUS_MAP = {
		healthy: [faHeart, "green"],
		running: [faCircleCheck, "green"],
		paused: [faCirclePause, "peach"],
		exited: [faCircleXmark, "red"],
		down: [faPowerOff, "text"],
		loading: [faHourglassStart, "sky"],
	} satisfies { [key in LuminaryStatus]: [IconDefinition, string] };

	let { status }: { status: LuminaryStatus } = $props();
	let [icon, color] = $derived(STATUS_MAP[status]);
</script>

<span style="color: var(--{color})" class="container">
	<Fa {icon} />
</span>

<style lang="scss">
	.container {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 1em;
	}
</style>
