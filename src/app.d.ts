// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
    namespace App {
        interface Error {
            id: string | undefined;
        }
        interface Locals {
            accessToken: string | undefined;
            refreshToken: string | undefined;
        }
        // interface PageData {}
        // interface PageState {}
        // interface Platform {}
    }
}

export { };
