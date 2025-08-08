import type { IpcResult } from "$lib/models/api";
import { invoke } from "@tauri-apps/api/core";
import { mapIpcResult } from "./api-utils";
import { emptyUserProfile, type UserProfile } from "./user";

export type SpaceRole = "owner" | "read" | "upload" | "modify";
export const spaceRoles = ["owner", "modify", "upload", "read"];

export interface SpaceMember {
    id: string;
    role: SpaceRole;
    space: SpaceData;
}

export interface SpaceData {
    id: string;
    name: string;
    description: string;
    picture_url: string;
}

export function emptySpaceMember() {
    return {
        id: "",
        role: "read",
        space: emptySpaceData(),
    } satisfies SpaceMember;
}

export function emptySpaceData() {
    return {
        id: "",
        name: "",
        description: "",
        picture_url: ""
    } satisfies SpaceData;
}

export interface SpaceUser {
    id: string;
    role: SpaceRole;
    user: UserProfile;
}

export function emptySpaceUser() {
    return {
        id: "",
        role: "read",
        user: emptyUserProfile(),
    } satisfies SpaceUser;
}

export async function getUserSpaces(token: string): Promise<IpcResult<SpaceMember[]>> {
    return mapIpcResult(invoke('get_user_spaces', { token }))
}

export async function getSpaceUsers(token: string, spaceId: string): Promise<IpcResult<SpaceUser[]>> {
    return mapIpcResult(invoke('get_space_users', { token, space_id: spaceId }))
}
