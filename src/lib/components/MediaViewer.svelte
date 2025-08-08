<script lang="ts">
    import {
        getStreamSignedUrl,
        getThumbnail,
        type FileMeta,
    } from "$lib/api/storage";
    import { fileExtension, fileName } from "$lib/utils";
    import { Image, Play, Video } from "@lucide/svelte";
    import { useClerkContext } from "svelte-clerk";
    import Badge from "./ui/badge/badge.svelte";
    import * as Dialog from "./ui/dialog";
    import { Skeleton } from "./ui/skeleton";

    let {
        files,
        index,
        streamUrls = $bindable(),
        spaceId,
    }: {
        files: FileMeta[];
        index: number;
        streamUrls: string[];
        spaceId: string;
    } = $props();

    const ctx = useClerkContext();

    let currentIndex = $state(index);
    let currentItem = $state(files[index]);
    let item = $derived(files[index]);

    let dialogOpen = $state(false);

    async function getStreamUrl(): Promise<string> {
        let url: string | undefined = streamUrls[currentIndex];
        if (url && url.length > 0) {
            return url;
        }
        const token = await ctx.session?.getToken();
        if (token) {
            const res = await getStreamSignedUrl(
                token,
                spaceId,
                currentItem.id,
            );
            if (res.type === "success") {
                streamUrls[currentIndex] = res.data.url;
                streamUrls = streamUrls;
                return res.data.url;
            }
        }

        return "";
    }

    async function getThumbnailImage() {
        const token = await ctx.session?.getToken();
        if (token) {
            const result = await getThumbnail(token, spaceId, item.id);
            if (result.type === "success") {
                const blob = new Blob([result.data]);
                return URL.createObjectURL(blob);
            }
        }
        return "";
    }
</script>

<Dialog.Root
    bind:open={dialogOpen}
    onOpenChange={(open) => {
        if (!open) {
            currentIndex = index;
            currentItem = files[index];
        }
    }}
>
    <Dialog.Trigger class="h-full object-cover hover:cursor-pointer">
        {#await getThumbnailImage() then thumbnailUrl}
            <img
                src={thumbnailUrl}
                alt={item.file_name}
                class="h-full object-cover rounded-lg transition-transform duration-200 group-hover:scale-105"
            />
        {/await}

        {#if item.media_type === "video"}
            <div
                class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-black/30 text-white text-sm p-2 transition-opacity truncate"
            >
                <Play />
            </div>
        {/if}
    </Dialog.Trigger>
    <Dialog.Content
        class="p-0 w-full h-full max-w-[95vw] max-h-[95vh] flex flex-col"
        trapFocus={false}
        onkeydown={(e) => {
            if (e.key === "ArrowUp" || e.key === "ArrowRight") {
                if (currentIndex < files.length - 1) {
                    currentIndex += 1;
                }
            }
            if (e.key === "ArrowDown" || e.key === "ArrowLeft") {
                if (currentIndex > 0) {
                    currentIndex -= 1;
                }
            }

            currentItem = files[currentIndex];
        }}
    >
        <div class="flex items-center h-fit gap-4 p-4 border-b">
            {#if currentItem.media_type === "video"}
                <Video class="text-primary" />
            {:else if currentItem.media_type === "image"}
                <Image class="text-primary" />
            {/if}

            <p class="text-sm font-medium truncate">
                {fileName(currentItem.file_name)}
            </p>
            <Badge>
                {fileExtension(currentItem.file_name).toLocaleUpperCase()}
            </Badge>
        </div>

        <div
            class="flex items-center justify-center w-full h-full max-w-full max-h-[85vh] rounded-lg px-4 group transition-all"
        >
            {#await getStreamUrl()}
                <Skeleton class="w-full h-full object-cover" />
            {:then url}
                {#if currentItem.media_type === "image"}
                    <img
                        loading="lazy"
                        src={url}
                        alt={currentItem.file_name}
                        class="w-auto h-full object-contain rounded-lg transition-all"
                    />
                {:else if currentItem.media_type === "video"}
                    <video
                        controls
                        class="w-full h-full object-contain rounded-lg transition-all"
                    >
                        <source src={url} />
                        <track kind="captions" />
                    </video>
                {/if}
            {/await}
        </div>
    </Dialog.Content>
</Dialog.Root>
