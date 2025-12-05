<script lang="ts">
    import { page } from "$app/state";
    import Separator from "$lib/components/ui/separator/separator.svelte";
    import { spaceMemberships } from "$lib/global-data";
    import { Routes } from "$lib/route";
    import { cn } from "$lib/utils";

    let spaceMembers = $derived($spaceMemberships);

    let selectedSpaceId = $derived<string | undefined>(page.params.spaceId);
    $effect(() => {
        console.log(selectedSpaceId);
    });
    let selectedSpaceMember = $derived(
        spaceMembers.find((m) => m.space.id === selectedSpaceId),
    );

    let isGallery = $derived(page.url.pathname.endsWith(Routes._Gallery));
</script>

<div
    class="sticky top-0 flex border-b border-dashed items-center justify-between"
>
    <div class="flex items-center">
        <a
            href={`${Routes.Cloud}/${selectedSpaceMember?.space.id}/${Routes._Gallery}`}
            class="flex flex-col justify-between h-full hover:bg-accent"
        >
            <p
                class={cn(
                    "py-2 px-4 text-sm",
                    isGallery ? "" : "text-muted-foreground",
                )}
            >
                Gallery
            </p>

            {#if isGallery}
                <Separator class="bg-primary h-5" orientation="horizontal" />
            {/if}
        </a>
        <a
            href={`${Routes.Cloud}/${selectedSpaceMember?.space.id}/${selectedSpaceMember?.folder}`}
            class="flex flex-col justify-between h-full hover:bg-accent"
        >
            <p
                class={cn(
                    "py-2 px-4 text-sm",
                    isGallery ? "text-muted-foreground" : "",
                )}
            >
                Browse
            </p>

            {#if !isGallery}
                <Separator class="bg-primary h-5" orientation="horizontal" />
            {/if}
        </a>
    </div>
</div>
