<script lang="ts">
    import { Routes } from "$lib/route";
    import { ChevronsUpDown, Plus } from "@lucide/svelte";
    import AppIcon from "./AppIcon.svelte";
    import { Button } from "./ui/button";
    import * as DropdownMenu from "./ui/dropdown-menu";
    import Separator from "./ui/separator/separator.svelte";
    import Skeleton from "./ui/skeleton/skeleton.svelte";
</script>

<header
    class="bg-background sticky top-0 z-50 h-10 flex w-full items-center border-b border-dashed select-none"
>
    <div data-tauri-drag-region class="h-full flex w-full items-center px-4">
        <Skeleton class="w-14" />

        <a
            href={Routes.Cloud}
            class="flex items-center gap-2 self-center mx-6 text-sm"
        >
            <AppIcon />
            Somachron
        </a>

        <Separator orientation="vertical" />

        <div class="flex items-center px-2 gap-2">
            <DropdownMenu.Root>
                <DropdownMenu.Trigger>
                    {#snippet child({ props })}
                        <Button {...props} variant="ghost">
                            <p
                                class="text-sm font-normal text-muted-foreground truncate text-ellipsis max-w-36"
                            >
                                owu2u329o23ofj2o3ifj2o3ifjoj23ofjo2jf23ojf2oi3fjo2ij32ofj23ofj2io3fjoi
                            </p>
                            <ChevronsUpDown />
                        </Button>
                    {/snippet}
                </DropdownMenu.Trigger>
                <DropdownMenu.Content class="w-56" align="start">
                    <DropdownMenu.Label>My Account</DropdownMenu.Label>
                    <DropdownMenu.Group>
                        <DropdownMenu.Item>
                            Profile
                            <DropdownMenu.Shortcut>⇧⌘P</DropdownMenu.Shortcut>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item>
                            Billing
                            <DropdownMenu.Shortcut>⌘B</DropdownMenu.Shortcut>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item>
                            Settings
                            <DropdownMenu.Shortcut>⌘S</DropdownMenu.Shortcut>
                        </DropdownMenu.Item>
                        <DropdownMenu.Item>
                            Keyboard shortcuts
                            <DropdownMenu.Shortcut>⌘K</DropdownMenu.Shortcut>
                        </DropdownMenu.Item>
                    </DropdownMenu.Group>
                    <DropdownMenu.Separator />
                    <DropdownMenu.Group>
                        <DropdownMenu.Item>Team</DropdownMenu.Item>
                        <DropdownMenu.Sub>
                            <DropdownMenu.SubTrigger
                                >Invite users</DropdownMenu.SubTrigger
                            >
                            <DropdownMenu.SubContent>
                                <DropdownMenu.Item>Email</DropdownMenu.Item>
                                <DropdownMenu.Item>Message</DropdownMenu.Item>
                                <DropdownMenu.Separator />
                                <DropdownMenu.Item>More...</DropdownMenu.Item>
                            </DropdownMenu.SubContent>
                        </DropdownMenu.Sub>
                        <DropdownMenu.Item>
                            New Team
                            <DropdownMenu.Shortcut>⌘+T</DropdownMenu.Shortcut>
                        </DropdownMenu.Item>
                    </DropdownMenu.Group>
                    <DropdownMenu.Separator />
                    <DropdownMenu.Item>GitHub</DropdownMenu.Item>
                    <DropdownMenu.Item>Support</DropdownMenu.Item>
                    <DropdownMenu.Item disabled>API</DropdownMenu.Item>
                    <DropdownMenu.Separator />
                    <DropdownMenu.Item>
                        Log out
                        <DropdownMenu.Shortcut>⇧⌘Q</DropdownMenu.Shortcut>
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>

            <Button size="icon-sm">
                <Plus />
            </Button>
        </div>

        <Separator orientation="vertical" />
    </div>
</header>
