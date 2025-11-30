<script lang="ts">
    import { goto } from "$app/navigation";
    import { validateAuth } from "$lib/api/auth";
    import AppIcon from "$lib/components/AppIcon.svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import { Routes } from "$lib/route";

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
    <title>Loading</title>
</svelte:head>

<section
    data-tauri-drag-region
    class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10"
>
    <div class="flex w-full max-w-sm flex-col gap-6">
        <a
            href={Routes.Cloud}
            class="flex items-center gap-2 self-center font-medium"
        >
            <AppIcon />
            Somachron
        </a>

        <div class="flex justify-center items-center h-full">
            <Spinner class="size-8" />
        </div>
    </div>
</section>
