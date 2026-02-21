<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { t } from "svelte-i18n";

    let {
        open = $bindable(false),
        onConfirm,
        onCancel,
        title,
        description,
    } = $props<{
        open: boolean;
        onConfirm: () => void;
        onCancel?: () => void;
        title?: string;
        description?: string;
    }>();

    function handleConfirm() {
        onConfirm();
        open = false;
    }

    function handleCancel() {
        if (onCancel) onCancel();
        open = false;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="max-w-[425px] w-full">
        <Dialog.Header>
            <Dialog.Title
                >{title ||
                    $t("general.confirmDeleteTitle") ||
                    "Are you sure?"}</Dialog.Title
            >
            <Dialog.Description>
                {description ||
                    $t("general.confirmDeleteDescription") ||
                    "This action cannot be undone."}
            </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
            <Button variant="outline" onclick={handleCancel}
                >{$t("general.cancel")}</Button
            >
            <Button variant="destructive" onclick={handleConfirm}>
                {$t("general.delete") || "Delete"}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
