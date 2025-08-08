<script lang="ts">
    import { Label } from "$lib/components/ui/label/index.js";
    import type { ApiEmpty } from "$lib/models/api.ts";
    import { isTypeApiEmpty } from "$lib/utils";
    import { CircleX, LoaderCircle, X } from "@lucide/svelte";
    import { toast } from "svelte-sonner";

    let { data } = $props();

    let err = $state<ApiEmpty | undefined>(undefined);

    $effect(() => {
        if (isTypeApiEmpty(data)) {
            err = data;
        }
    });
</script>

<svelte:head>
    <title>Somachron | Auth callback</title>
</svelte:head>

<div
    class="flex h-screen w-screen items-center justify-center place-items-center"
>
    <div class="flex flex-col gap-4 w-full items-center justify-center">
        {#if err}
            <CircleX class="size-12" />
            <Label class="text-base">{err.message}</Label>
            {#if err.message.includes("Not allowed")}
                <Label class="text-sm">Contact developer with your email</Label>
            {/if}
        {:else}
            <LoaderCircle class="animate-spin size-12" />
        {/if}
    </div>
</div>
