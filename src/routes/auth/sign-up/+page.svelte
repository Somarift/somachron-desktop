<script lang="ts">
    import { setupClient } from "$lib/api/auth";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Routes } from "$lib/route";
    import { CloudMoon } from "@lucide/svelte";
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
</script>

<svelte:head>
    <title>Somachron | Sign Up</title>
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

        <Card.Root>
            <Card.Header class="text-center">
                <Card.Title class="text-xl">Welcome</Card.Title>
                <Card.Description>Sign up with your email</Card.Description>
            </Card.Header>
            <Card.Content>
                <form>
                    <div class="grid gap-6">
                        <div class="grid gap-6">
                            <Label>Name</Label>
                            <div class="grid grid-cols-2 gap-3">
                                <Input
                                    id="first_name"
                                    type="text"
                                    placeholder="First Name"
                                    required
                                />

                                <Input
                                    id="last_name"
                                    type="text"
                                    placeholder="Last Name"
                                    required
                                />
                            </div>
                            <div class="grid gap-3">
                                <Label for="email">Email</Label>
                                <Input
                                    id="email"
                                    type="email"
                                    placeholder="m@example.com"
                                    required
                                />
                            </div>
                            <Button type="submit" class="w-full">Login</Button>
                        </div>
                        <div class="text-center text-sm">
                            Already have an account?
                            <a
                                href={Routes.SignIn}
                                class="underline underline-offset-4"
                            >
                                Sign in
                            </a>
                        </div>
                    </div>
                </form>
            </Card.Content>
        </Card.Root>
    </div>
</div>
