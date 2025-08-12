<script lang="ts">
    import { getUserSpaces } from "$lib/api/space";
    import { emptyUserProfile, getUserProfile } from "$lib/api/user";
    import AppSidebar from "$lib/components/AppSidebar.svelte";
    import Titlebar from "$lib/components/Titlebar.svelte";
    import * as Sidebar from "$lib/components/ui/sidebar";
    import { spaces, user } from "$lib/states";
    import { validateApiFront } from "$lib/utils";

    let { children } = $props();

    $effect(() => {
        const fetchData = async () => {
            $user = await validateApiFront(
                getUserProfile(),
                emptyUserProfile(),
            );

            $spaces = await validateApiFront(getUserSpaces(), []);
        };
        fetchData();
    });
</script>

<svelte:head>
    <title>Somachron</title>
</svelte:head>

<div class="[--header-height:calc(--spacing(11))]">
    <Sidebar.Provider class="flex flex-col">
        <Titlebar />

        <div class="flex flex-1">
            <AppSidebar />

            <Sidebar.Inset class="max-h-screen overflow-auto">
                {@render children?.()}
            </Sidebar.Inset>
        </div>
    </Sidebar.Provider>
</div>
