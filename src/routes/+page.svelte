<script lang="ts">
    import { goto } from "$app/navigation";
    import { validateAuth } from "$lib/api/auth";
    import { Routes } from "$lib/route";
    import { CloudMoon, Loader2 } from "@lucide/svelte";

    $effect(() => {
        const runValidation = async () => {
            let result = await validateAuth();
            console.log(result);
            let path =
                result.type === "success"
                    ? result.data === "app"
                        ? Routes.Cloud
                        : Routes.SignIn
                    : Routes.SignIn;

            goto(path, {
                invalidateAll: true,
                replaceState: true,
            });
        };
        runValidation();
    });
</script>

<svelte:head>
    <title>Home</title>
</svelte:head>

<div
    data-tauri-drag-region
    class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10"
>
    <div class="flex w-full max-w-sm flex-col gap-6">
        <a
            href={Routes.Cloud}
            class="flex items-center gap-2 self-center font-medium"
        >
            <div
                class="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md"
            >
                <CloudMoon class="text-background" />
            </div>
            Somachron
        </a>

        <div class="flex justify-center items-center h-full">
            <Loader2 class="w-8 h-8 text-primary animate-spin" />
        </div>
    </div>
</div>
