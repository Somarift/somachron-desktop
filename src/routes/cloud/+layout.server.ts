import { Routes } from "$lib/route";
import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoadEvent } from "./$types";

export async function load({ locals }: LayoutServerLoadEvent): Promise<{}> {
    const session = locals.accessToken;
    if (!session) {
        throw redirect(302, Routes.SignIn);
    }
    return {};
}
