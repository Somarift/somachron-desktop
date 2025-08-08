import type { ApiEmpty, IpcResult } from "$lib/models/api";

export async function mapIpcResult<T>(invoke: Promise<T>): Promise<IpcResult<T>> {
    return invoke.then((d) => {
        let decoder = new TextDecoder();
        let data = decoder.decode(d as ArrayBuffer);
        return { type: "success", data: JSON.parse(data) } satisfies IpcResult<T>;
    })
        .catch((err) => ({ type: "error", error: err as ApiEmpty } satisfies IpcResult<T>));
}
