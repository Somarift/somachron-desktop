<script lang="ts">
    // import { PUBLIC_CLERK_PUBLISHABLE_KEY } from "$env/static/public";
    import { Toaster } from "$lib/components/ui/sonner";
    import { ModeWatcher } from "mode-watcher";
    // import { ClerkProvider } from "svelte-clerk/client";
    import "../app.css";

    let { children } = $props();
</script>

<ModeWatcher />
<Toaster richColors={true} position="bottom-center" closeButton={true} />

<!-- <ClerkProvider publishableKey={PUBLIC_CLERK_PUBLISHABLE_KEY}> -->
<main>
    {@render children()}
</main>
<!-- </ClerkProvider> -->
