import { clsx, type ClassValue } from "clsx";
import { toast } from "svelte-sonner";
import { twMerge } from "tailwind-merge";
import type { ApiEmpty } from "./models/api";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

export function isType<T>(value: any, field: string): value is T {
    if (value === null || typeof value !== "object") {
        return false;
    }
    return field in value;
}

export function isTypeApiEmpty(value: any): value is ApiEmpty {
    return isType<ApiEmpty>(value, "req_id");
}

export async function validateApiFront<T>(data: Promise<T>, def: T, suppressErr: boolean = false, msg: string = ""): Promise<T> {
    let res = await data;
    if (isTypeApiEmpty(res)) {
        if (!suppressErr) {
            let message = msg.length > 0 ? msg + " - " + res.message : res.message;
            toast.error(message);
        }
        return def;
    }
    return res;
}

export function humanFileSize(bytes: number, dp = 1) {
    const thresh = 1000;

    if (Math.abs(bytes) < thresh) {
        return bytes + ' B';
    }

    const units = ['kB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    let u = -1;
    const r = 10 ** dp;

    do {
        bytes /= thresh;
        ++u;
    } while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1);

    return bytes.toFixed(dp) + ' ' + units[u];
}

export function fileName(name: string): string {
    let idx = name.lastIndexOf('.');
    if (idx === -1) {
        return name;
    }

    return name.slice(0, idx)
}

export function fileExtension(name: string): string {
    let idx = name.lastIndexOf('.');
    if (idx === -1) {
        return "";
    }

    return name.slice(idx + 1)
}

export function capitalize(text: string): string {
    if (!text || text.length === 0) {
        return text;
    }

    return text.charAt(0).toUpperCase() + text.substring(1);
}
