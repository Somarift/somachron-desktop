<script lang="ts">
    import { Label } from "./ui/label";
    import { Separator } from "./ui/separator";
    import * as Sidebar from "./ui/sidebar";

    let {
        data,
        children,
    }: {
        data: { icon: any; label: string };
        children?: any;
    } = $props();
</script>

<div class="sticky top-0 z-10 bg-background">
    <div class="flex items-center gap-2 py-2 px-4 w-full border-b">
        <Sidebar.Trigger class="-ml-1 px-1" />
        <Separator orientation="vertical" class="mr-2 h-4" />
        <div class="w-full flex items-center justify-between">
            <div class="flex gap-4 items-center">
                <data.icon class="size-4" />
                <Label class="font-medium">{data.label}</Label>
            </div>

            {@render children?.()}
        </div>
    </div>
</div>
