<script lang="ts">
    import { page } from "$app/state";
    import { Routes } from "$lib/route";
    import { spaces } from "$lib/states";
    import { BookImage, ChevronRightIcon } from "@lucide/svelte";
    import CreateSpaceDialog from "./CreateSpaceDialog.svelte";
    import * as Collapsible from "./ui/collapsible";
    import * as Sidebar from "./ui/sidebar";

    let open = $state(true);

    const sidebar = Sidebar.useSidebar();
</script>

<Sidebar.Group>
    <Sidebar.GroupLabel>Cloud</Sidebar.GroupLabel>
    <Sidebar.Menu>
        <Collapsible.Root {open} class="group/collapsible">
            {#snippet child({ props })}
                <Sidebar.MenuItem {...props}>
                    <Collapsible.Trigger>
                        {#snippet child({ props })}
                            <Sidebar.MenuButton
                                {...props}
                                tooltipContent="Spaces"
                                isActive={page.url.pathname.startsWith(
                                    Routes.Cloud,
                                )}
                            >
                                <a
                                    href={Routes.Cloud}
                                    class="flex items-center gap-2"
                                    {...props}
                                    onclick={() => {
                                        if (sidebar.isMobile) {
                                            sidebar.toggle();
                                        }
                                    }}
                                >
                                    <BookImage class="size-4" />
                                    <span>Spaces</span>
                                </a>
                                <ChevronRightIcon
                                    class="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90"
                                />
                            </Sidebar.MenuButton>
                        {/snippet}
                    </Collapsible.Trigger>
                    <Collapsible.Content>
                        <Sidebar.MenuSub>
                            {#each $spaces as subItem (subItem.id)}
                                <Sidebar.MenuSubItem>
                                    <Sidebar.MenuSubButton
                                        isActive={page.url.pathname
                                            .replace(Routes.Cloud, "")
                                            .startsWith(`/${subItem.space.id}`)}
                                    >
                                        {#snippet child({ props })}
                                            <a
                                                href={`${Routes.Cloud}/${subItem.space.id}`}
                                                {...props}
                                                onclick={() => {
                                                    if (sidebar.isMobile) {
                                                        sidebar.toggle();
                                                    }
                                                }}
                                            >
                                                <span>
                                                    {subItem.space.name}
                                                </span>
                                            </a>
                                        {/snippet}
                                    </Sidebar.MenuSubButton>
                                </Sidebar.MenuSubItem>
                            {/each}
                            <Sidebar.MenuSubItem>
                                <Sidebar.MenuSubButton>
                                    {#snippet child({ props })}
                                        <CreateSpaceDialog
                                            isSidebar={true}
                                            {...props}
                                        />
                                    {/snippet}
                                </Sidebar.MenuSubButton>
                            </Sidebar.MenuSubItem>
                        </Sidebar.MenuSub>
                    </Collapsible.Content>
                </Sidebar.MenuItem>
            {/snippet}
        </Collapsible.Root>
    </Sidebar.Menu>
</Sidebar.Group>
