import { writable } from "svelte/store";
import type { SpaceMember } from "./api/space";
import { emptyUserProfile } from "./api/user";

export const userProfile = writable(emptyUserProfile());
export const spaceMemberships = writable([] as SpaceMember[]);
