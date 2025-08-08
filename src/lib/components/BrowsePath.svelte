<script lang="ts">
    import { page } from "$app/state";
    import * as Breadcrumb from "$lib/components/ui/breadcrumb";
    import { Routes } from "$lib/route";

    let {
        limit = 0,
        className = "",
        replacements = [],
    }: {
        limit?: number;
        className?: string;
        replacements?: { idx: number; value: string }[];
    } = $props();

    let basePath = Routes.Cloud;

    function getPath(): string[] {
        var path = page.url.pathname
            .replace(basePath, "")
            .split("/")
            .filter(Boolean);
        if (limit > 0) {
            path = path.slice(0, limit);
        }
        return path;
    }

    function makeReplacements(paths: string[]): string[] {
        let visiblePaths = [...paths];
        replacements.forEach(({ idx, value }) => {
            if (idx >= 0 && idx < visiblePaths.length) {
                visiblePaths[idx] = value;
            }
        });
        return visiblePaths;
    }

    let path = $derived(getPath());
    let visiblePath = $derived(makeReplacements(path));
</script>

<Breadcrumb.Root class={className}>
    <Breadcrumb.List class="bg-muted w-fit rounded-md px-2 py-1">
        <Breadcrumb.Separator>/</Breadcrumb.Separator>
        {#each visiblePath as item, i}
            {#if i === visiblePath.length - 1}
                <Breadcrumb.Page>
                    <div class="max-w-36 truncate text-ellipsis">
                        {item.replaceAll("%20", " ")}
                    </div>
                </Breadcrumb.Page>
            {:else}
                <Breadcrumb.Link
                    href={`${page.url.origin}/${basePath.replaceAll("/", "")}/${path
                        .slice(0, i + 1)
                        .join("/")}`}
                >
                    <div class="max-w-36 truncate text-ellipsis">
                        {item.replaceAll("%20", " ")}
                    </div>
                </Breadcrumb.Link>
            {/if}
            {#if i !== visiblePath.length - 1}
                <Breadcrumb.Separator>/</Breadcrumb.Separator>
            {/if}
        {/each}
    </Breadcrumb.List>
</Breadcrumb.Root>
