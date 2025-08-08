import { Routes } from "$lib/route";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoadEvent } from "./$types";

export async function load({ locals }: PageServerLoadEvent) {
    const session = locals.accessToken;
    if (session) {
        throw redirect(302, Routes.Cloud);
    }
}
