<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { ChevronLeft, ChevronRight, Save, X } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let {
        currentStep = 1,
        totalSteps = 5,
        isSubmitting = false,
        onPrev = () => {},
        onNext = () => {},
        onSubmit = () => {},
        onClose = () => {},
    } = $props<{
        currentStep: number;
        totalSteps: number;
        isSubmitting: boolean;
        onPrev: () => void;
        onNext: () => void;
        onSubmit: () => void;
        onClose: () => void;
    }>();

    const isFirstStep = $derived(currentStep === 1);
    const isLastStep = $derived(currentStep === totalSteps);
</script>

<div
    class="flex items-center justify-between border-t border-zinc-800 bg-black/40 px-6 py-4 mt-auto"
>
    <div class="flex gap-2">
        <Button
            variant="ghost"
            onclick={onClose}
            disabled={isSubmitting}
            class="text-zinc-500 hover:text-white"
        >
            <X class="w-4 h-4 mr-2" />
            {$t("trades.wizard.summary.cancel")}
        </Button>
    </div>

    <div class="flex gap-2">
        {#if !isFirstStep}
            <Button
                variant="outline"
                onclick={onPrev}
                disabled={isSubmitting}
                class="border-zinc-800 text-zinc-300"
            >
                <ChevronLeft class="w-4 h-4 mr-2" />
                {$t("trades.wizard.summary.prev")}
            </Button>
        {/if}

        {#if !isLastStep}
            <Button
                onclick={onNext}
                disabled={isSubmitting}
                class="bg-primary text-black font-bold"
            >
                {$t("trades.wizard.summary.next")}
                <ChevronRight class="w-4 h-4 ml-2" />
            </Button>
        {:else}
            <Button
                onclick={onSubmit}
                disabled={isSubmitting}
                class="bg-green-600 hover:bg-green-700 text-white font-bold"
            >
                <Save class="w-4 h-4 mr-2" />
                {isSubmitting
                    ? $t("trades.wizard.summary.saving")
                    : $t("trades.wizard.summary.save_trade")}
            </Button>
        {/if}
    </div>
</div>
