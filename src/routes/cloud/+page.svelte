<script lang="ts">
    import CreateSpaceDialog from "$lib/components/CreateSpaceDialog.svelte";
    import * as Empty from "$lib/components/ui/empty";
    import * as Item from "$lib/components/ui/item";
    import { spaceMemberships } from "$lib/global-data";
    import { Routes } from "$lib/route";
    import { Images } from "@lucide/svelte";

    let spaceMembers = $derived($spaceMemberships);
</script>

<div class="flex flex-col p-3 gap-4">
    {#if spaceMembers.length > 0}
        <div class="w-full flex justify-end">
            <CreateSpaceDialog isMenu={false} />
        </div>

        <div class="flex flex-wrap gap-2">
            {#each spaceMembers as member (member.id)}
                <Item.Root variant="outline" class="w-2xs">
                    {#snippet child({ props })}
                        <a
                            href={`${Routes.Cloud}/${member.space.id}/${member.folder}`}
                            {...props}
                        >
                            <Item.Media variant="icon">
                                <Images />
                            </Item.Media>
                            <Item.Content>
                                <Item.Title>{member.space.name}</Item.Title>
                                <Item.Description>
                                    {member.space.description ||
                                        "No description"}
                                </Item.Description>
                            </Item.Content>
                            <Item.Actions />
                        </a>
                    {/snippet}
                </Item.Root>
            {/each}
        </div>
    {:else}
        <Empty.Root class="border border-dashed">
            <Empty.Header>
                <Empty.Media variant="icon">
                    <Images />
                </Empty.Media>
                <Empty.Title>No Cloud Spaces</Empty.Title>
                <Empty.Description>
                    Create your space and upload files to access them anywhere.
                </Empty.Description>
            </Empty.Header>
            <Empty.Content>
                <CreateSpaceDialog isMenu={false} />
            </Empty.Content>
        </Empty.Root>
    {/if}
</div>
