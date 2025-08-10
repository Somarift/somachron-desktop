import type { IpcResult } from "$lib/models/api";
import { invoke } from "@tauri-apps/api/core";
import { mapIpc } from "./api-utils";

export async function setupClient(): Promise<IpcResult<any>> {
    return mapIpc(invoke('setup_client', {}), (a) => a as any);
}

export async function signIn(email: string): Promise<IpcResult<any>> {
    return mapIpc(invoke('sign_in', { email }), (a) => a as any);
}

export async function attemptFactor(code: string): Promise<IpcResult<any>> {
    return mapIpc(invoke('attempt_factor', { code }), (a) => a as any);
}
