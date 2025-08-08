<script lang="ts">
    import { enhance } from "$app/forms";
    import { invalidateAll } from "$app/navigation";
    import { Plus } from "@lucide/svelte";
    import type { ActionResult } from "@sveltejs/kit";
    import { toast } from "svelte-sonner";
    import * as AlertDialog from "./ui/alert-dialog";
    import { Input } from "./ui/input";
    import { buttonVariants } from "./ui/button";

    let { isSidebar, ...props } = $props();

    let spaceName = $state("");
    let spaceDesc = $state("");
    let loading = $state(false);
    let createSpaceForm: any = $state();
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

    async function handleOnComplete(result: ActionResult) {
        if (result.type === "failure") {
            let err = result.data;
            toast.error(err?.error || "Something went wrong");
        }
        if (result.type === "success") {
            toast.success("Space created");
        }

        await invalidateAll();
        loading = false;
        dialogOpen = false;
    }
</script>

<form
    bind:this={createSpaceForm}
    action="/?/createSpace"
    method="POST"
    use:enhance={({ formData }) => {
        loading = true;
        validateFolderName();

        formData.set("name", spaceName.trim());
        formData.set("desc", spaceDesc.trim());

        return async ({ update, result }) => {
            update({
                invalidateAll: true,
                reset: true,
            });
            await handleOnComplete(result);
        };
    }}
></form>

<AlertDialog.Root
    bind:open={dialogOpen}
    onOpenChange={(open) => {
        if (!open) {
            spaceName = "";
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
                    if (createSpaceForm) {
                        createSpaceForm.dispatchEvent(new Event("submit"));
                    }
                }}
            >
                Create
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
