<script lang="ts">
    import { goto } from "$app/navigation";
    import { attemptFactor, setupClient, signIn } from "$lib/api/auth";
    import AppIcon from "$lib/components/AppIcon.svelte";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import {
        Field,
        FieldDescription,
        FieldGroup,
        FieldLabel,
    } from "$lib/components/ui/field";
    import { Input } from "$lib/components/ui/input";
    import * as InputOTP from "$lib/components/ui/input-otp";
    import { Spinner } from "$lib/components/ui/spinner";
    import { Routes } from "$lib/route";
    import { toast } from "svelte-sonner";

    $effect(() => {
        const runSetup = async () => {
            let result = await setupClient();
            if (result.type === "error") {
                toast.error("Error setting up client: " + result.error.message);
            }
        };
        runSetup();
    });

    let loading = $state(false);
    let signInState = $state<"code" | "email">("email");

    async function attempSignIn(email: string) {
        if (loading) {
            return;
        }

        loading = true;

        let result = await signIn(email.trim());
        if (result.type === "success") {
            signInState = "code";
        } else {
            toast.error(result.error.message);
        }

        loading = false;
    }

    async function attempOtp(otp: string) {
        loading = true;

        let result = await attemptFactor(otp.trim());
        if (result.type === "success") {
            toast.success("Logged in !");
            goto(Routes.Cloud, { invalidateAll: true, replaceState: true });
        } else {
            toast.error(result.error.message);
        }

        loading = false;
    }

    function resetState() {
        signInState = "email";
        loading = false;
    }
</script>

<div
    data-tauri-drag-region
    class="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10"
>
    <div class="flex w-full max-w-sm flex-col gap-6">
        <a href="##" class="flex items-center gap-2 self-center font-medium">
            <AppIcon />
            Somachron
        </a>

        <div class="flex flex-col gap-6">
            <Card.Root>
                <Card.Header class="text-center">
                    <Card.Title class="text-xl">Welcome back</Card.Title>
                    <Card.Description>Login with your email</Card.Description>
                </Card.Header>
                <Card.Content>
                    <form
                        onsubmit={(e) => {
                            let formData = new FormData(e.currentTarget);

                            if (signInState === "email") {
                                let email =
                                    formData.get("email")?.toString() || "";
                                attempSignIn(email);
                            } else {
                                let code =
                                    formData.get("otp")?.toString() || "";
                                attempOtp(code);
                            }
                        }}
                    >
                        <FieldGroup>
                            <Field>
                                <FieldLabel for="email">Email</FieldLabel>
                                <Input
                                    id="email"
                                    type="email"
                                    name="email"
                                    placeholder="m@example.com"
                                    required
                                    disabled={signInState === "code"}
                                />
                            </Field>

                            {#if signInState === "code"}
                                <Field>
                                    <FieldLabel for="otp">
                                        Verification code
                                    </FieldLabel>
                                    <InputOTP.Root
                                        maxlength={6}
                                        id="otp"
                                        name="otp"
                                        required
                                    >
                                        {#snippet children({ cells })}
                                            <InputOTP.Group
                                                class="flex items-center w-full justify-between *:data-[slot=input-otp-slot]:rounded-md *:data-[slot=input-otp-slot]:border"
                                            >
                                                {#each cells as cell (cell)}
                                                    <InputOTP.Slot {cell} />
                                                {/each}
                                            </InputOTP.Group>
                                        {/snippet}
                                    </InputOTP.Root>

                                    <FieldDescription>
                                        Enter the 6-digit code sent to your
                                        email.
                                    </FieldDescription>
                                </Field>

                                <Field
                                    orientation="horizontal"
                                    class="flex items-center"
                                >
                                    <Button
                                        type="button"
                                        variant="destructive"
                                        disabled={loading}
                                        onclick={resetState}
                                        class="flex-1"
                                    >
                                        Cancel
                                    </Button>
                                    <Button
                                        type="submit"
                                        disabled={loading}
                                        class="flex-1"
                                    >
                                        {#if loading}
                                            <Spinner class="size-4" />
                                        {/if}
                                        Verify
                                    </Button>
                                </Field>
                            {:else}
                                <Field>
                                    <Button type="submit" disabled={loading}>
                                        {#if loading}
                                            <Spinner class="size-4" />
                                        {/if}
                                        Login
                                    </Button>
                                    <FieldDescription class="text-center">
                                        Don't have an account?
                                        <a href="##"> Sign up </a>
                                    </FieldDescription>
                                </Field>
                            {/if}
                        </FieldGroup>
                    </form>
                </Card.Content>
            </Card.Root>
            <FieldDescription class="px-6 text-center">
                By clicking continue, you agree to our
                <a href="##">Terms of Service</a>
                and <a href="##">Privacy Policy</a>.
            </FieldDescription>
        </div>
    </div>
</div>
