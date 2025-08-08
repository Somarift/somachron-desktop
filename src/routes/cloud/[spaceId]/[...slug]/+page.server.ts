import { Routes } from "$lib/route";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoadEvent } from "./$types";

export async function load({ locals, params }: PageServerLoadEvent): Promise<{}> {
    let token = locals.accessToken;
    if (!token) {
        throw redirect(302, Routes.SignIn);
    }

    return {};
}
