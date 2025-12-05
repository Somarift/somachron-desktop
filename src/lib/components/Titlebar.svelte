<script lang="ts">
    import { spaceMemberships, userProfile } from "$lib/global-data";
    import { Routes } from "$lib/route";
    import { capitalize } from "$lib/utils";
    import { ChevronsUpDown, Images, LogOut, Plus } from "@lucide/svelte";
    import AppIcon from "./AppIcon.svelte";
    import * as Avatar from "./ui/avatar";
    import { Badge } from "./ui/badge";
    import { Button } from "./ui/button";
    import * as DropdownMenu from "./ui/dropdown-menu";
    import Skeleton from "./ui/skeleton/skeleton.svelte";
    import CreateSpaceDialog from "./CreateSpaceDialog.svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";

    let user = $derived($userProfile);
    let spaceMembers = $derived($spaceMemberships);

    let selectedSpaceId = $derived<string | undefined>(page.params.spaceId);
    let selectedSpaceMember = $derived(
        spaceMembers.find((m) => m.space.id === selectedSpaceId),
    );
</script>

<header
    class="bg-background sticky top-0 z-50 h-10 flex w-full items-center select-none"
>
    <div
        data-tauri-drag-region
        class="h-full flex w-full items-center justify-start px-1"
    >
        <Skeleton class="w-20" />

        <a
            href={Routes.Cloud}
            class="flex items-center gap-2 self-center mx-6 text-sm"
        >
            <AppIcon />
            Somachron
        </a>

        <DropdownMenu.Root>
            <DropdownMenu.Trigger>
                {#snippet child({ props })}
                    <Button
                        {...props}
                        variant="outline"
                        size="sm"
                        class={selectedSpaceMember ? "" : "border-dashed"}
                    >
                        <Images />
                        {#if selectedSpaceMember}
                            <p
                                class="text-sm font-normal truncate text-ellipsis max-w-36"
                            >
                                {selectedSpaceMember.space.name}
                            </p>
                            <Badge class="text-xs font-normal rounded-sm mx-1">
                                {capitalize(selectedSpaceMember.role)}
                            </Badge>
                        {:else}
                            <p
                                class="text-sm font-normal text-muted-foreground truncate text-ellipsis"
                            >
                                Select space
                            </p>
                        {/if}

                        <ChevronsUpDown />
                    </Button>
                {/snippet}
            </DropdownMenu.Trigger>
            <DropdownMenu.Content class="w-56" align="start">
                <DropdownMenu.Label>Spaces</DropdownMenu.Label>
                <DropdownMenu.Group>
                    {#each spaceMembers as membership (membership.id)}
                        <DropdownMenu.Item
                            class="justify-between"
                            onclick={() => {
                                goto(
                                    `${Routes.Cloud}/${membership.space.id}/${membership.folder}`,
                                );
                            }}
                        >
                            {membership.space.name}

                            <Badge class="text-xs font-normal rounded-sm">
                                {capitalize(membership.role)}
                            </Badge>
                        </DropdownMenu.Item>
                    {/each}
                </DropdownMenu.Group>
                <DropdownMenu.Separator />
                <CreateSpaceDialog isMenu={true} />
            </DropdownMenu.Content>
        </DropdownMenu.Root>

        <div class="w-full"></div>

        <DropdownMenu.Root>
            <DropdownMenu.Trigger>
                {#snippet child({ props })}
                    <Button
                        {...props}
                        variant="ghost"
                        size="icon-sm"
                        class="px-1 mx-1 rounded-full"
                    >
                        <Avatar.Root>
                            <Avatar.Image
                                class="p-1 rounded-full"
                                src={user.picture_url}
                                alt={user.given_name}
                            />
                            <Avatar.Fallback>
                                {user.given_name.charAt(0)}
                            </Avatar.Fallback>
                        </Avatar.Root>
                    </Button>
                {/snippet}
            </DropdownMenu.Trigger>
            <DropdownMenu.Content class="w-56" align="start">
                <DropdownMenu.Group>
                    <DropdownMenu.Item>
                        <Avatar.Root>
                            <Avatar.Image
                                src={user.picture_url}
                                alt={user.given_name}
                            />
                            <Avatar.Fallback>
                                {user.given_name.charAt(0)}
                            </Avatar.Fallback>
                        </Avatar.Root>
                        <div>
                            <p class="text-sm">{user.given_name}</p>
                            <p class="text-xs text-muted-foreground">
                                {user.email}
                            </p>
                        </div>
                    </DropdownMenu.Item>
                </DropdownMenu.Group>
                <DropdownMenu.Separator />
                <DropdownMenu.Item>
                    <LogOut />
                    Log out
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>
</header>
