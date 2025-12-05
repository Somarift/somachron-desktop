<script lang="ts">
    import { page } from "$app/state";

    $effect(() => {
        console.log(page.params.folderId);
    });
</script>

<div>Browse here</div>
