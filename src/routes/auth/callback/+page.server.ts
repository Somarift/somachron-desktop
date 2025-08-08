// import { syncAuth } from "$lib/api/auth.server";
import { Routes } from "$lib/route";
import { isTypeApiEmpty } from "$lib/utils";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoadEvent } from "./$types";

export async function load({ locals }: PageServerLoadEvent) {
    const token = locals.accessToken;
    if (!token) {
        console.log("Missing session");
        throw redirect(401, Routes.SignIn);
    }

    var authSuccess = false;
    try {
        // let _ = await syncAuth(token);
        authSuccess = true;
    } catch (e) {
        if (isTypeApiEmpty(e)) {
            if (e.status === 401 && e.message.includes("Not allowed")) {
                return { ...e };
            }
        } else {
            authSuccess = false;
        }
    }

    if (authSuccess) {
        throw redirect(302, Routes.Cloud);
    } else {
        throw redirect(302, Routes.SignIn);
    }
};
