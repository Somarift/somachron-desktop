<script lang="ts">
    import { page } from "$app/state";
    import { getSpaceUsers, type SpaceUser } from "$lib/api/space";
    import { listDirItems, type FileEntry } from "$lib/api/storage";
    import BrowsePath from "$lib/components/BrowsePath.svelte";
    import CreateFolderDialog from "$lib/components/CreateFolderDialog.svelte";
    import DeletePathDialog from "$lib/components/DeletePathDialog.svelte";
    import PageHeader from "$lib/components/PageHeader.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as ContextMenu from "$lib/components/ui/context-menu/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import { spaces } from "$lib/states.js";
    import { cn, validateApiFront } from "$lib/utils";
    import { Folder, Grid2X2, Images, Rows3 } from "@lucide/svelte";
    import { onDestroy, onMount } from "svelte";
    import { useClerkContext } from "svelte-clerk/client";
    import { folderViewState } from "./state.svelte.js";

    const ctx = useClerkContext();

    let currentPath = $derived(page.params.slug);
    let spaceId = $derived(page.params.spaceId);
    let spaceUsers = $state([] as SpaceUser[]);
    let entries = $state([] as FileEntry[]);
    let currentSpace = $derived($spaces.find((s) => s.space.id === spaceId));

    $effect(() => {
        currentPath = page.params.slug;

        const fetchData = async () => {
            const token = await ctx.session?.getToken();
            if (token) {
                spaceUsers = await validateApiFront(
                    getSpaceUsers(token, spaceId),
                    [],
                );

                entries = await validateApiFront(
                    listDirItems(token, spaceId, `/${currentPath}`),
                    [],
                );
            }
        };
        fetchData();
    });

    let view: any = $state();
    let folders = $derived(entries.filter((e) => e.tag === "dir"));
    let files = $derived(entries.filter((e) => e.tag === "file"));
    let filesCount = $state(15);
    let viewedFiles = $derived(files.slice(0, filesCount));
    let streamUrls = $derived(files.map((_) => ""));

    function loadMoreFiles() {
        if (files.length < filesCount + 15) {
            filesCount = files.length;
        } else {
            filesCount += 15;
            viewedFiles = files.slice(0, filesCount);
        }
    }

    function onScrollListener() {
        if (view.scrollTop + view.clientHeight >= view.scrollHeight) {
            loadMoreFiles();
        }
    }

    onMount(() => {
        if (view) {
            view.addEventListener("scroll", onScrollListener);
        }
    });

    onDestroy(() => {
        entries = [];
        if (view) {
            view.removeEventListener("scroll", onScrollListener);
        }
    });
</script>

<svelte:head>
    <title>Somachron | {currentSpace?.space.name} Space</title>
</svelte:head>

<section bind:this={view} class="overflow-auto">
    <PageHeader
        data={{
            icon: Images,
            label: currentSpace?.space.name || "Media",
        }}
    >
        <div class="flex items-center gap-2">
            <!-- <SpaceUsersSheet
                space={currentSpace?.space || emptySpaceData()}
                members={spaceMembers}
                {platformUsers}
                currentUser={user}
            /> -->
            <CreateFolderDialog
                disabled={currentSpace?.role !== "owner" &&
                    currentSpace?.role !== "modify"}
                onCreate={() => {
                    entries = [];
                }}
            />
            <!-- <UploadDialog
                disabled={currentSpace?.role !== "owner" &&
                    currentSpace?.role !== "modify"}
            /> -->
        </div>
    </PageHeader>

    <div class="w-full flex items-center justify-between px-4">
        <BrowsePath
            className="py-2"
            replacements={[
                {
                    idx: 0,
                    value: "root",
                },
            ]}
        />

        <p class="text-sm text-muted-foreground">
            {files.length + folders.length} items
        </p>
    </div>

    <div class="flex flex-col gap-4 p-4">
        {#if folders.length > 0}
            <div class="flex justify-between items-center">
                <Label>Folders</Label>

                <div>
                    <Button
                        size="icon"
                        variant="outline"
                        onclick={() => {
                            folderViewState.layout = "grid";
                        }}
                    >
                        <Grid2X2 />
                    </Button>
                    <Button
                        size="icon"
                        variant="outline"
                        onclick={() => {
                            folderViewState.layout = "list";
                        }}
                    >
                        <Rows3 />
                    </Button>
                </div>
            </div>
            <div
                class={cn(
                    "flex-wrap gap-4",
                    folderViewState.layout === "list" ? "flex-col" : "flex",
                )}
            >
                {#each folders as item}
                    <ContextMenu.Root>
                        <ContextMenu.Trigger>
                            <a
                                class={cn(
                                    "flex items-center gap-2 p-4 min-w-56 hover:bg-muted text-sm",
                                    folderViewState.layout === "list"
                                        ? "border-b"
                                        : "rounded-md border",
                                )}
                                href={`${page.url.pathname}/${item.name}`}
                            >
                                <Folder />
                                {item.name}
                            </a>
                        </ContextMenu.Trigger>
                        <ContextMenu.Content>
                            <ContextMenu.Item closeOnSelect={false}>
                                <DeletePathDialog
                                    disabled={currentSpace?.role !== "owner" &&
                                        currentSpace?.role !== "modify"}
                                    path={`${currentPath}/${item.name}`}
                                />
                            </ContextMenu.Item>
                        </ContextMenu.Content>
                    </ContextMenu.Root>
                {/each}
            </div>
        {/if}

        {#key files}
            {#if files.length > 0}
                <Label>Files</Label>
                <div class="flex flex-wrap gap-4">
                    {#each viewedFiles as item, i}
                        <ContextMenu.Root>
                            <ContextMenu.Trigger
                                class="relative group rounded-lg overflow-hidden cursor-pointer"
                            >
                                <!-- <MediaViewer
                                    bind:streamUrls
                                    files={files.map((f) => f.file)}
                                    index={i}
                                    {spaceId}
                                /> -->
                                <div
                                    class="absolute bottom-0 left-0 right-0 bg-black/30 text-white text-sm px-2 py-1 opacity-0 group-hover:opacity-100 transition-opacity truncate"
                                >
                                    {item.file.file_name}
                                </div>
                            </ContextMenu.Trigger>
                            <ContextMenu.Content>
                                <ContextMenu.Item closeOnSelect={false}>
                                    <DeletePathDialog
                                        disabled={currentSpace?.role !==
                                            "owner" &&
                                            currentSpace?.role !== "modify"}
                                        path={`${currentPath}/${item.file.file_name}`}
                                    />
                                </ContextMenu.Item>
                            </ContextMenu.Content>
                        </ContextMenu.Root>
                    {/each}
                </div>
            {/if}
        {/key}
    </div>
</section>
