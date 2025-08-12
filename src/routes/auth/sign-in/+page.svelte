<script lang="ts">
    import { goto } from "$app/navigation";
    import { attemptFactor, setupClient, signIn } from "$lib/api/auth";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Input } from "$lib/components/ui/input";
    import * as InputOTP from "$lib/components/ui/input-otp";
    import { Label } from "$lib/components/ui/label";
    import { Routes } from "$lib/route";
    import { CloudMoon, Info, Loader } from "@lucide/svelte";
    import { REGEXP_ONLY_DIGITS } from "bits-ui";
    import { toast } from "svelte-sonner";

    let loading = $state(false);
    let resendDisabled = $state(true);
    let email = $state("");
    let otp = $state("");
    let signInError = $state("");
    let signInState = $state<"login" | "code">("login");

    $effect(() => {
        const runSetup = async () => {
            let result = await setupClient();
            if (result.type === "error") {
                toast.error("Error setting up client: " + result.error.message);
            }
        };
        runSetup();
    });

    async function attempSignIn() {
        loading = true;
        signInError = "";

        let result = await signIn(email.trim());
        if (result.type === "success") {
            signInState = "code";
            resendDisabled = true;
            setTimeout(() => {
                resendDisabled = false;
            }, 20 * 1000);
        } else {
            signInError = result.error.message;
        }

        loading = false;
    }

    async function attempOtp() {
        loading = true;

        let result = await attemptFactor(otp.trim());
        if (result.type === "success") {
            toast.success("Logged in !");
            goto(Routes.Cloud, { invalidateAll: true, replaceState: true });
        } else {
            signInError = result.error.message;
        }

        loading = false;
    }

    function resetState() {
        signInState = "login";
        email = "";
        otp = "";
        resendDisabled = true;
    }
</script>

<svelte:head>
    <title>Somachron | Log in</title>
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
                <CloudMoon class="size-5 text-background" />
            </div>
            Somachron
        </a>

        <Card.Root>
            <Card.Header class="text-center">
                <Card.Title class="text-xl">Welcome back</Card.Title>
                <Card.Description>Login with your email</Card.Description>
            </Card.Header>
            <Card.Content>
                <div class="grid gap-6">
                    <div class="grid gap-6 transition-all">
                        <div class="grid gap-3">
                            <Label for="email">Email</Label>
                            <Input
                                id="email"
                                type="email"
                                placeholder="user@email.com"
                                bind:value={email}
                                required
                                disabled={loading || signInState === "code"}
                            />

                            {#if signInError && signInError.length > 0}
                                <div class="flex items-center gap-2">
                                    <Info class="text-destructive size-4" />
                                    <Label class="text-destructive">
                                        {signInError}
                                    </Label>
                                </div>
                            {/if}
                        </div>

                        {#if signInState === "code"}
                            <div class="grid gap-3 items-center justify-center">
                                <InputOTP.Root
                                    maxlength={6}
                                    bind:value={otp}
                                    writingsuggestions={false}
                                    pattern={REGEXP_ONLY_DIGITS}
                                    onComplete={() => {
                                        attempOtp();
                                    }}
                                >
                                    {#snippet children({ cells })}
                                        <InputOTP.Group>
                                            {#each cells.slice(0, 3) as cell (cell)}
                                                <InputOTP.Slot {cell} />
                                            {/each}
                                        </InputOTP.Group>
                                        <InputOTP.Separator />
                                        <InputOTP.Group>
                                            {#each cells.slice(3, 6) as cell (cell)}
                                                <InputOTP.Slot {cell} />
                                            {/each}
                                        </InputOTP.Group>
                                    {/snippet}
                                </InputOTP.Root>

                                <div
                                    class="flex items-center gap-2 justify-center"
                                >
                                    <Button
                                        class="text-xs text-muted-foreground hover:underline hover:cursor-pointer"
                                        variant="link"
                                        onclick={(e) => {
                                            e.preventDefault();
                                            attempSignIn();
                                        }}
                                        disabled={resendDisabled}
                                    >
                                        Didn't receive a code ? Resend
                                    </Button>
                                </div>
                            </div>
                        {/if}

                        {#if signInState === "code"}
                            <Button
                                type="submit"
                                class="w-full flex items-center hover:cursor-pointer"
                                variant="destructive"
                                disabled={loading}
                                onclick={() => {
                                    resetState();
                                }}
                            >
                                Cancel
                            </Button>
                        {:else}
                            <Button
                                type="submit"
                                class="w-full flex items-center hover:cursor-pointer"
                                disabled={loading}
                                onclick={() => {
                                    if (signInState === "login") {
                                        attempSignIn();
                                    }
                                }}
                            >
                                {#if loading}
                                    <Loader class="animate-spin" />
                                {/if}
                                Login
                            </Button>
                        {/if}
                    </div>
                    <div class="text-center text-sm">
                        Don&apos;t have an account?
                        <a
                            href={Routes.SignUp}
                            class="underline underline-offset-4"
                        >
                            Sign up
                        </a>
                    </div>
                </div>
            </Card.Content>
        </Card.Root>
    </div>
</div>
