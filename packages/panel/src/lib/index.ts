import { faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
import { faCircleXmark } from "@fortawesome/free-regular-svg-icons";
import { addToast } from "../routes/Toaster.svelte";

export { openDialog, closeDialog } from "../routes/Dialog.svelte";

export * as api from "./api";

export { isMobile } from "../routes/+layout.svelte";

export function trim(str: string, maxLength: number) {
	if (str.length <= maxLength) return str;
	return str.slice(0, maxLength - 3) + "...";
}

export function warn(message: string, details?: string | string[]) {
	addToast({ data: { icon: faTriangleExclamation, color: "peach", title: trim(message, 40), details } });
}

export function error(message: string, details?: string | string[]) {
	addToast({ data: { icon: faCircleXmark, color: "red", title: trim(message, 40), details } });
}

export function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export class Backoff {
	public currentDelay: number;
	public initialDelay: number;
	public maxDelay: number;

	public constructor(initialDelay: number = 1000, maxDelay: number = 30000) {
		this.currentDelay = initialDelay;
		this.initialDelay = initialDelay;
		this.maxDelay = maxDelay;
	}

	public reset() {
		this.currentDelay = this.initialDelay;
	}

	public async wait() {
		await sleep(this.currentDelay);
		this.currentDelay = Math.min(this.currentDelay * 2, this.maxDelay);
	}
}
