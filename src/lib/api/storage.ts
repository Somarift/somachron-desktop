import type { IpcResult } from "$lib/models/api";
import { invoke } from "@tauri-apps/api/core";
import { mapIpc, mapIpcResult } from "./api-utils";

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

export async function listDirItems(spaceId: string, path: string): Promise<IpcResult<FileEntry[]>> {
    return mapIpcResult(invoke('list_dir_items', { space_id: spaceId, path }));
}

export async function createFolder(spaceId: string, path: string): Promise<IpcResult<any>> {
    return mapIpc(invoke('create_folder', { space_id: spaceId, path }), (a) => a as any);
}

export async function deletePath(spaceId: string, path: string): Promise<IpcResult<any>> {
    return mapIpc(invoke('delete_path', { space_id: spaceId, path }), (a) => a as any);
}

export async function getThumbnail(spaceId: string, fileId: string): Promise<IpcResult<ArrayBuffer>> {
    return mapIpc(invoke('get_thumbnail', { space_id: spaceId, file_id: fileId }), (a) => a);
}

export async function getStreamSignedUrl(spaceId: string, fileId: string): Promise<IpcResult<SignedUrl>> {
    return mapIpcResult(invoke('get_stream_signed_url', { space_id: spaceId, file_id: fileId }));
}
