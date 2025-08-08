import type { ApiEmpty, IpcResult } from "$lib/models/api";

export async function mapIpcResult<T>(invoke: Promise<ArrayBuffer>): Promise<IpcResult<T>> {
    return mapIpc(invoke, (d) => {
        let decoder = new TextDecoder();
        let data = decoder.decode(d);
        return JSON.parse(data);
    });
}

export async function mapIpc<T>(invoke: Promise<ArrayBuffer>, transform: (arr: ArrayBuffer) => T): Promise<IpcResult<T>> {
    return invoke.then((buf) => ({ type: "success", data: transform(buf) } satisfies IpcResult<T>))
        .catch((err) => ({ type: "error", error: err as ApiEmpty } satisfies IpcResult<T>));
}
