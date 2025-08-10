<script lang="ts">
    import { browser } from "$app/environment";
    import { goto } from "$app/navigation";
    import { Routes } from "$lib/route";
    import { onMount } from "svelte";

    onMount(() => {
        if (browser) {
            goto(Routes.SignIn);
        }
    });
</script>

<svelte:head>
    <title>Home</title>
    <meta name="description" content="Svelte demo app" />
</svelte:head>
