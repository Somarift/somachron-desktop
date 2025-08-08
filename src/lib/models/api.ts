export interface ApiEmpty {
    status: number;
    message: string;
    req_id: string;
}

export type IpcResult<T> = { type: 'success'; data: T } | { type: 'error'; error: ApiEmpty };
