import { writable } from "svelte/store";
import type { SpaceMember } from "./api/space";
import { emptyUserProfile } from "./api/user";

export const user = writable(emptyUserProfile());
export const spaces = writable([] as SpaceMember[]);
