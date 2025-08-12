import type { IpcResult } from "$lib/models/api";
import { invoke } from "@tauri-apps/api/core";
import { mapIpcResult } from "./api-utils";

export interface UserProfile {
    id: string;
    create_at: string;
    updated_at: string;
    given_name: string;
    picture_url: string;
}

export function emptyUserProfile(): UserProfile {
    return {
        id: "",
        create_at: "",
        updated_at: "",
        given_name: "",
        picture_url: ""
    } satisfies UserProfile;
}

export interface PlatformUser {
    id: string;
    create_at: string;
    given_name: string;
    picture_url: string;
}

export function emptyPlatformUser() {
    return {
        id: "",
        create_at: "",
        given_name: "",
        picture_url: ""
    } satisfies PlatformUser;
}

export async function getUserProfile(): Promise<IpcResult<UserProfile>> {
    return mapIpcResult(invoke('get_user_profile', {}));
}
