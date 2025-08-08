<script lang="ts">
    import { Routes } from "$lib/route";
    import { CloudMoon, Loader2 } from "@lucide/svelte";
    import { ClerkLoading, UserButton } from "svelte-clerk";
    import Skeleton from "./ui/skeleton/skeleton.svelte";
</script>

<header
    class="bg-background sticky top-0 z-50 flex w-full items-center border-b select-none"
>
    <div
        data-tauri-drag-region
        class="h-(--header-height) flex w-full justify-between items-center gap-2 px-4"
    >
        <Skeleton class="w-36" />

        <div class="flex justify-center items-center gap-2">
            <div class="bg-primary rounded-md p-[2px]">
                <CloudMoon class="text-background size-4" />
            </div>
            <p class="font-medium text-sm">Somachron</p>
        </div>

        <div class="flex justify-end items-center">
            <ClerkLoading>
                <div class="flex justify-center items-center h-full">
                    <Loader2 class="size-7 text-primary animate-spin" />
                </div>
            </ClerkLoading>
            <UserButton showName={true} afterSignOutUrl={Routes.SignIn} />
        </div>
    </div>
</header>
