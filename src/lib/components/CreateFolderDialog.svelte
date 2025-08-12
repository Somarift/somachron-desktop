<script lang="ts">
    import { invalidateAll } from "$app/navigation";
    import { page } from "$app/state";
    import { createFolder } from "$lib/api/storage";
    import { FolderPlus } from "@lucide/svelte";
    import { toast } from "svelte-sonner";
    import * as AlertDialog from "./ui/alert-dialog";
    import { buttonVariants } from "./ui/button";
    import { Input } from "./ui/input";

    let { disabled, onCreate }: { disabled: boolean; onCreate: () => void } =
        $props();

    let uploadPath = $derived(`/${page.params.slug}`);

    let folderName = $state("");
    let loading = $state(false);
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

    async function createFolderForm() {
        loading = true;

        let result = await createFolder(
            page.params.spaceId,
            encodeURI(`${uploadPath}/${folderName.trim()}`),
        );
        if (result.type === "error") {
            toast.error(
                `[${result.error.status}] ${result.error.message} - Failed to create folder`,
            );
        } else {
            onCreate();
            toast.success("Folder created");
        }

        await invalidateAll();
        folderName = "";
        loading = false;
        dialogOpen = false;
    }
</script>

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
                    createFolderForm();
                }}
            >
                Create
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
