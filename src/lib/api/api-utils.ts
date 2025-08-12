import type { ApiEmpty, IpcResult } from "$lib/models/api";

export async function mapIpcResult<T>(invoke: Promise<ArrayBuffer>): Promise<IpcResult<T>> {
    return mapIpc(invoke, (buf) => {
        let data = decodeText(buf);
        return JSON.parse(data);
    });
}

export async function mapIpc<T>(invoke: Promise<ArrayBuffer>, transform: (buf: ArrayBuffer) => T): Promise<IpcResult<T>> {
    return invoke.then((buf) => ({ type: "success", data: transform(buf) } satisfies IpcResult<T>))
        .catch((err) => ({ type: "error", error: err as ApiEmpty } satisfies IpcResult<T>));
}

export function decodeText(buf: ArrayBuffer): string {
    let decoder = new TextDecoder();
    return decoder.decode(buf);
}
