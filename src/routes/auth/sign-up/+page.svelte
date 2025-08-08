<script lang="ts">
    import { Routes } from "$lib/route";
    import { CloudMoon, Loader2 } from "@lucide/svelte";
    import { ClerkLoading, SignUp } from "svelte-clerk";
</script>

<svelte:head>
    <title>Somachron | Sign Up</title>
</svelte:head>

<div
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

        <div>
            <ClerkLoading>
                <div class="flex justify-center items-center h-full">
                    <Loader2 class="w-8 h-8 text-primary animate-spin" />
                </div>
            </ClerkLoading>
        </div>

        <SignUp signInUrl={Routes.SignIn} redirectUrl={Routes.AuthCallback} />
    </div>
</div>
