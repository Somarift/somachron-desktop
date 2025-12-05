<script lang="ts">
    import { cn } from "$lib/utils";
    import { CloudMoon } from "@lucide/svelte";

    let { className }: { className?: string } = $props();
</script>

<div
    class={cn(
        "bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-sm p-0.5",
        className,
    )}
>
    <CloudMoon class="text-white" />
</div>
