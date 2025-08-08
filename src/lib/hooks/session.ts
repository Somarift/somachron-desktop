import { CLERK_SECRET_KEY } from "$env/static/private";
import { verifyToken } from "svelte-clerk/server";

export const verifySession = async (token: string) => {
    const claims = await verifyToken(token, {
        secretKey: CLERK_SECRET_KEY,
    });
    return claims;
}
