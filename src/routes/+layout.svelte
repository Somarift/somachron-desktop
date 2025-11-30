<script lang="ts">
    import { ModeWatcher } from "mode-watcher";
    import { Toaster } from "$lib/components/ui/sonner";
    import "../app.css";

    let { children } = $props();
</script>

<ModeWatcher />
<Toaster richColors={true} position="bottom-center" closeButton={true} />

<div data-tauri-drag-region>
    {@render children()}
</div>
