<script lang="ts">
    import { getUserSpaces } from "$lib/api/space";
    import { emptyUserProfile, getUserProfile } from "$lib/api/user";
    import Titlebar from "$lib/components/Titlebar.svelte";
    import * as Sidebar from "$lib/components/ui/sidebar";
    import { spaceMemberships, userProfile } from "$lib/global-data";
    import { validateApiFront } from "$lib/utils";

    let { children } = $props();

    $effect(() => {
        const fetchData = async () => {
            $userProfile = await validateApiFront(
                getUserProfile(),
                emptyUserProfile(),
            );

            $spaceMemberships = await validateApiFront(getUserSpaces(), []);
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

        <Sidebar.Inset class="max-h-screen overflow-auto">
            {@render children?.()}
        </Sidebar.Inset>
    </Sidebar.Provider>
</div>
