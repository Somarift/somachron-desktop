<script lang="ts">
    import { cn, validateApiFront } from "$lib/utils";
    import { ChevronsUpDownIcon, MoonIcon, SunIcon } from "@lucide/svelte";
    import { UserButton } from "svelte-clerk";
    import * as Sidebar from "./ui/sidebar";
    import { toast } from "svelte-sonner";
    import { Routes } from "$lib/route";
    import type { UserProfile } from "$lib/api/user";
    import * as DropdownMenu from "./ui/dropdown-menu";
    import { buttonVariants } from "./ui/button";
    import { resetMode, setMode } from "mode-watcher";

    let { user }: { user: UserProfile } = $props();

    const sidebar = Sidebar.useSidebar();
</script>

<Sidebar.Menu>
    <Sidebar.MenuItem>
        <Sidebar.MenuButton
            size="lg"
            class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground hover:bg-sidebar"
        >
            <UserButton afterSignOutUrl={Routes.SignIn} />
            <button
                class="grid flex-1 text-left text-sm leading-tight"
                onclick={() => toast.info("Click on profile icon")}
            >
                <span class="truncate font-medium">
                    {user.given_name}
                </span>
                <span class="truncate text-xs">
                    {user.email}
                </span>
            </button>
            <DropdownMenu.Trigger
                class={cn(
                    buttonVariants({
                        variant: "outline",
                        size: "icon",
                        class: "shadow-none border-none p-2 h-full w-fit hover:cursor-pointer",
                    }),
                    sidebar.state === "collapsed" ? "hidden" : "",
                )}
            >
                <SunIcon
                    class="h-[1rem] w-[1rem] rotate-0 scale-100 !transition-all dark:-rotate-90 dark:scale-0"
                />
                <MoonIcon
                    class="absolute h-[1rem] w-[1rem] rotate-90 scale-0 !transition-all dark:rotate-0 dark:scale-100"
                />
                <span class="sr-only">Toggle theme</span>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
                <DropdownMenu.Item onclick={() => setMode("light")}>
                    Light
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => setMode("dark")}>
                    Dark
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => resetMode()}>
                    System
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </Sidebar.MenuButton>
    </Sidebar.MenuItem>
</Sidebar.Menu>
