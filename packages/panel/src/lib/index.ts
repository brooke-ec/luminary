import { faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
import { faCircleXmark } from "@fortawesome/free-regular-svg-icons";
import { addToast } from "../routes/Toaster.svelte";

export * as api from "./api";

export function warn(title: string, details?: string | string[]) {
	addToast({ data: { icon: faTriangleExclamation, color: "peach", title, details } });
}

export function error(title: string, details?: string | string[]) {
	addToast({ data: { icon: faCircleXmark, color: "red", title, details } });
}
