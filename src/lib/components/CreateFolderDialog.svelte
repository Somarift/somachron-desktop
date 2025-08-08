<script lang="ts">
    import { enhance } from "$app/forms";
    import { invalidateAll } from "$app/navigation";
    import { page } from "$app/state";
    import { FolderPlus } from "@lucide/svelte";
    import type { ActionResult } from "@sveltejs/kit";
    import { toast } from "svelte-sonner";
    import * as AlertDialog from "./ui/alert-dialog";
    import { buttonVariants } from "./ui/button";
    import { Input } from "./ui/input";
    import type { ApiEmpty } from "$lib/models/api";

    let { disabled }: { disabled: boolean } = $props();

    let uploadPath = $derived(`/${page.params.slug}`);

    let folderName = $state("");
    let loading = $state(false);
    let createFolderForm: any = $state();
    let dialogOpen = $state(false);

    function validateFolderName() {
        folderName = folderName.replaceAll("/", "");
        folderName = folderName.replaceAll("\\", "");
        if (folderName.startsWith("tmp")) {
            folderName = folderName.replaceAll("tmp", "");
        }
        if (folderName.length > 64) {
            folderName = folderName.slice(0, 63);
        }
    }

    async function handleOnComplete(result: ActionResult) {
        if (result.type === "failure") {
            let data = result.data;
            let err = data?.err as ApiEmpty | undefined;
            toast.error(err?.message || "Something went wrong");
        }
        if (result.type === "success") {
            toast.success("Folder created");
        }

        await invalidateAll();
        folderName = "";
        loading = false;
        dialogOpen = false;
    }
</script>

<form
    bind:this={createFolderForm}
    action="?/createFolder"
    method="POST"
    use:enhance={({ formData }) => {
        loading = true;
        validateFolderName();

        formData.set("folder", `${uploadPath}/${folderName.trim()}`);

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
            folderName = "";
            loading = false;
        }
    }}
>
    <AlertDialog.Trigger
        class={buttonVariants({ variant: "outline", size: "icon" })}
        {disabled}
    >
        <FolderPlus />
    </AlertDialog.Trigger>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>Create new folder</AlertDialog.Title>
            <AlertDialog.Description>
                Path <code class="font-mono">{uploadPath || "/"}</code>
            </AlertDialog.Description>
        </AlertDialog.Header>
        <div class="flex flex-col gap-2">
            <Input
                placeholder="Folder name"
                value={folderName}
                oninput={({ target }) => {
                    if (target) {
                        folderName = (target as HTMLInputElement).value;
                        validateFolderName();
                    }
                }}
            />
        </div>
        <AlertDialog.Footer>
            <AlertDialog.Cancel disabled={loading}>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action
                disabled={folderName.length === 0 || loading}
                onclick={() => {
                    if (createFolderForm) {
                        createFolderForm.dispatchEvent(new Event("submit"));
                    }
                }}
            >
                Create
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
