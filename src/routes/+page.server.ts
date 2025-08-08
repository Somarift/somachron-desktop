import { Routes } from "$lib/route";
import { redirect } from "@sveltejs/kit";

export async function load() {
    redirect(302, Routes.Cloud);
}

// export const actions: Actions = {
//     createSpace: async ({ locals, request }) => {
//         let token = locals.accessToken;
//         if (!token) {
//             return fail(401, { err: "Unauthorized" });
//         }

//         let formData = await request.formData();
//         let name = formData.get("name")?.toString() || "";
//         let desc = formData.get("desc")?.toString() || "";

//         try {
//             const _ = await createSpace(token, name, desc);
//             return {};
//         } catch (e) {
//             return fail(400, { err: `${e}` });
//         }
//     }
// };
