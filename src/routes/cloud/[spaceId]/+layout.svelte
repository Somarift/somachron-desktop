<script lang="ts">
    import SpaceHeader from "$lib/components/SpaceHeader.svelte";

    let { children } = $props();
</script>

<SpaceHeader />

{@render children?.()}
