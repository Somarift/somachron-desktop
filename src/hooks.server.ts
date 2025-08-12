import { Routes } from "$lib/route";
import { type Handle } from "@sveltejs/kit";

const PROTECTED_ROUTES = [Routes.Cloud];

export const handle: Handle = async ({ event, resolve }) => {

    // const sessionCookie = event.cookies.get(SESSION_TOKEN);
    // const isProtected = PROTECTED_ROUTES.find((p) => event.url.pathname.startsWith(p));

    // if (sessionCookie) {
    //     try {
    //         const _ = await verifySession(sessionCookie);
    //         event.locals.accessToken = sessionCookie;
    //     } catch (e) {
    //         if (isProtected) {
    //             const fullSignInUrl = new URL(Routes.SignIn, event.url.origin);
    //             return Response.redirect(fullSignInUrl.toString() + '?redirectUrl=' + event.url.pathname);
    //         }
    //     }
    // }

    return await resolve(event);
};
