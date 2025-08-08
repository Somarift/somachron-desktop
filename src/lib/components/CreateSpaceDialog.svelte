<script lang="ts">
    import { invalidateAll } from "$app/navigation";
    import { createSpace, getUserSpaces } from "$lib/api/space";
    import { spaces } from "$lib/states";
    import { validateApiFront } from "$lib/utils";
    import { Plus } from "@lucide/svelte";
    import { useClerkContext } from "svelte-clerk";
    import { toast } from "svelte-sonner";
    import * as AlertDialog from "./ui/alert-dialog";
    import { buttonVariants } from "./ui/button";
    import { Input } from "./ui/input";

    let { isSidebar, ...props } = $props();

    const ctx = useClerkContext();

    let spaceName = $state("");
    let spaceDesc = $state("");
    let loading = $state(false);
    let dialogOpen = $state(false);

    function validateFolderName() {
        spaceName = spaceName.replaceAll("/", "");
        spaceName = spaceName.replaceAll("\\", "");
        if (spaceName.startsWith("tmp")) {
            spaceName = spaceName.replaceAll("tmp", "");
        }
        if (spaceName.length > 64) {
            spaceName = spaceName.slice(0, 63);
        }
    }

    async function createSpaceForm() {
        loading = true;

        const token = await ctx.session?.getToken();
        if (token) {
            let result = await createSpace(
                token,
                spaceName.trim(),
                spaceDesc.trim(),
            );
            if (result.type === "error") {
                toast.error(
                    `[${result.error.status}] ${result.error.message} - Failed to create space`,
                );
            } else {
                $spaces = await validateApiFront(getUserSpaces(token), []);
                toast.success("Space created");
            }
        } else {
            toast.error("No auth token");
        }

        await invalidateAll();
        loading = false;
        dialogOpen = false;
    }
</script>

<AlertDialog.Root
    bind:open={dialogOpen}
    onOpenChange={(open) => {
        if (!open) {
            spaceName = "";
            spaceDesc = "";
            loading = false;
        }
    }}
>
    {#if isSidebar}
        <AlertDialog.Trigger {...props}>
            <Plus />
            Create Space
        </AlertDialog.Trigger>
    {:else}
        <AlertDialog.Trigger
            class={buttonVariants({ size: "sm", class: "h-7" })}
        >
            <Plus />
            Create Space
        </AlertDialog.Trigger>
    {/if}
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>Create new space</AlertDialog.Title>
            <AlertDialog.Description>
                Your shareable space
            </AlertDialog.Description>
        </AlertDialog.Header>
        <div class="flex flex-col gap-2">
            <Input
                placeholder="Name"
                value={spaceName}
                oninput={({ target }) => {
                    if (target) {
                        spaceName = (target as HTMLInputElement).value;
                        validateFolderName();
                    }
                }}
            />
            <Input
                placeholder="Description"
                value={spaceDesc}
                oninput={({ target }) => {
                    if (target) {
                        spaceDesc = (target as HTMLInputElement).value;
                    }
                }}
            />
        </div>
        <AlertDialog.Footer>
            <AlertDialog.Cancel disabled={loading}>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action
                disabled={spaceName.length === 0 || loading}
                onclick={() => {
                    createSpaceForm();
                }}
            >
                Create
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
