<script lang="ts">
    import CreateSpaceDialog from "$lib/components/CreateSpaceDialog.svelte";
    import PageHeader from "$lib/components/PageHeader.svelte";
    import { Routes } from "$lib/route.js";
    import { spaces } from "$lib/states";
    import { BookImage, Images } from "@lucide/svelte";
</script>

<section>
    <PageHeader
        data={{
            icon: BookImage,
            label: "Spaces",
        }}
    >
        <div class="flex items-center gap-2">
            <CreateSpaceDialog isSidebar={false} />
        </div>
    </PageHeader>

    <div class="flex flex-wrap w-full gap-4 p-4">
        {#each $spaces as member}
            <a
                class="flex items-center min-w-56 border p-4 gap-4 rounded-md"
                href={`${Routes.Cloud}/${member.space.id}`}
            >
                <Images class="size-6" />
                <div class="flex flex-col">
                    <p>
                        {member.space.name}
                    </p>
                    <p class="text-muted-foreground text-sm truncate">
                        {member.space.description}
                    </p>
                </div>
            </a>
        {/each}
    </div>
</section>
