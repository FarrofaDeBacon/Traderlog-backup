<script lang="ts">
    import NewTradeWizard from "$lib/components/trades/NewTradeWizard.svelte";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

    const closeWindow = () => {
        getCurrentWebviewWindow().close();
    };
</script>

<div class="h-screen w-full bg-background p-4 overflow-auto">
    <NewTradeWizard detached={true} close={closeWindow} onsave={closeWindow} />
</div>
