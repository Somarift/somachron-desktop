<script lang="ts">
    import CloudSidebar from "./CloudSidebar.svelte";
    import * as DropdownMenu from "./ui/dropdown-menu";
    import * as Sidebar from "./ui/sidebar";
</script>

<Sidebar.Root
    collapsible="icon"
    variant="sidebar"
    class="top-(--header-height) h-[calc(100svh-var(--header-height))]! select-none"
>
    <DropdownMenu.Root>
        <Sidebar.Content>
            <CloudSidebar />
        </Sidebar.Content>
        <Sidebar.Footer>
            <!-- <UserSidebar {user} /> -->
        </Sidebar.Footer>
        <Sidebar.Rail />
    </DropdownMenu.Root>
</Sidebar.Root>
