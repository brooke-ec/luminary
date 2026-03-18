import { faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
import { faCircleXmark } from "@fortawesome/free-regular-svg-icons";
import { addToast } from "../routes/Toaster.svelte";

export { openDialog, closeDialog } from "../routes/Dialog.svelte";

export * as api from "./api";

export { isMobile } from "../routes/+layout.svelte";

/**
 * Trims {@paramstr} to {@param maxLength}, adding ellipses if it was too long.
 * @param str The string to trim
 * @param maxLength The maximum number of characters to allow before trimming.
 * @returns The trimmed string.
 */
export function trim(str: string, maxLength: number) {
	if (str.length <= maxLength) return str;
	return str.slice(0, maxLength - 3) + "...";
}

/**
 * Displays a warning toast to the user.
 * @param message The message to display on the toast.
 * @param details Any extra details to show in the toast modal.
 */
export function warn(message: string, details?: string | string[]) {
	addToast({ data: { icon: faTriangleExclamation, color: "peach", title: trim(message, 40), details } });
}

/**
 * Displays an error toast to the user.
 * @param message The message to display on the toast.
 * @param details Any extra details to show in the toast modal.
 */
export function error(message: string, details?: string | string[]) {
	addToast({ data: { icon: faCircleXmark, color: "red", title: trim(message, 40), details } });
}

/**
 * A simple async sleep function that resolves after {@param ms} milliseconds.
 * @param ms The number of milliseconds to sleep for.
 */
export function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * A exponential backoff utility class used for managing retry attempts.
 */
export class Backoff {
	/**
	 * The current delay before the next retry attempt, in milliseconds.
	 */
	public currentDelay: number;

	/**
	 * The initial delay, set when the class is instantiated or reset.
	 */
	public initialDelay: number;

	/**
	 * The maximum delay allowed.
	 */
	public maxDelay: number;

	public constructor(initialDelay: number = 1000, maxDelay: number = 30 * 1000) {
		this.currentDelay = initialDelay;
		this.initialDelay = initialDelay;
		this.maxDelay = maxDelay;
	}

	/**
	 * Resets the backoff delay to the initial value.
	 */
	public reset() {
		this.currentDelay = this.initialDelay;
	}

	/**
	 * Waits for the current delay duration, and then doubles it.
	 */
	public async wait() {
		await sleep(this.currentDelay);
		this.currentDelay = Math.min(this.currentDelay * 2, this.maxDelay);
	}
}
