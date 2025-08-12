<script lang="ts">
    import { invalidateAll } from "$app/navigation";
    import { page } from "$app/state";
    import { deletePath } from "$lib/api/storage";
    import { Delete } from "@lucide/svelte";
    import { toast } from "svelte-sonner";
    import * as AlertDialog from "./ui/alert-dialog";
    import { buttonVariants } from "./ui/button";

    let { path, disabled }: { path: string; disabled: boolean } = $props();

    let loading = $state(false);
    let dialogOpen = $state(false);

    async function deletePathForm() {
        loading = true;

        let result = await deletePath(page.params.spaceId, encodeURI(path));
        if (result.type === "error") {
            toast.error(
                `[${result.error.status}] ${result.error.message} - Failed to create folder`,
            );
        } else {
            toast.success("Folder created");
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
            loading = false;
        }
    }}
>
    <AlertDialog.Trigger
        class="text-red-500 flex items-center justify-between w-full font-semibold"
        {disabled}
    >
        Delete
        <Delete class="size-5 text-red-500" />
    </AlertDialog.Trigger>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>Delete item ?</AlertDialog.Title>
            <AlertDialog.Description>
                Are you sure you want to delete {path}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel disabled={loading}>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action
                class={buttonVariants({ variant: "destructive" })}
                disabled={loading}
                onclick={() => {
                    deletePathForm();
                }}
            >
                Delete
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
