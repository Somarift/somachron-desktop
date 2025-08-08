import type { IpcResult } from "$lib/models/api";
import { invoke } from "@tauri-apps/api/core";
import { mapIpcResult, mapIpcVoid } from "./api-utils";

export interface SignedUrl {
    url: string;
}

export const FsTag = 'fs::';

export type MediaType = 'image' | 'video';

export interface FileData {
    id: string;
    file_name: string;
    metadata: { [key: string]: any },
    r2_path: string,
    file_size: number,
    thumbnail_path: string;
    member: string;
    media_type: MediaType;
}

export interface FileMeta {
    id: string;
    file_name: string;
    user: string | undefined;
    media_type: MediaType;
}

export type FileEntry = | { tag: 'dir'; name: string } | { tag: 'file'; file: FileMeta };

export async function listDirItems(token: string, spaceId: string, path: string): Promise<IpcResult<FileEntry[]>> {
    return mapIpcResult(invoke('list_dir_items', { token, space_id: spaceId, path }));
}

export async function createFolder(token: string, spaceId: string, path: string): Promise<IpcResult<any>> {
    return mapIpcVoid(invoke('create_folder', { token, space_id: spaceId, path }));
}

export async function deletePath(token: string, spaceId: string, path: string): Promise<IpcResult<any>> {
    return mapIpcVoid(invoke('delete_path', { token, space_id: spaceId, path }));
}
