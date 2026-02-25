<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import * as Select from "$lib/components/ui/select";
    import { Textarea } from "$lib/components/ui/textarea";
    import Slider from "$lib/components/ui/slider/slider.svelte";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { Brain, Sparkles, Loader2, MessageSquare } from "lucide-svelte";
    import { llmService } from "$lib/services/llmService";

    let { open = $bindable(false) } = $props();

    let selectedDate = $state(new Date().toISOString().split("T")[0]);
    let emotionalStateId = $state<string>("");
    let emotionalIntensity = $state(5);
    let journalNotes = $state("");

    let isAnalyzing = $state(false);
    let aiFeedback = $state("");

    const emotionalStates = $derived(settingsStore.emotionalStates);

    // Watch for date changes to load existing data
    $effect(() => {
        if (open && selectedDate) {
            const existingEntry =
                settingsStore.getJournalEntryByDate(selectedDate);
            if (existingEntry) {
                emotionalStateId = existingEntry.emotional_state_id || "";
                emotionalIntensity = existingEntry.intensity;
                journalNotes = existingEntry.content;
            } else {
                // Only reset if we switched dates, don't wipe while editing?
                // Currently simplified to reset if no entry found.
                emotionalStateId = "";
                emotionalIntensity = 5;
                journalNotes = "";
            }
        }
    });

    async function save() {
        if (!emotionalStateId && !journalNotes) {
            toast.error("Preencha pelo menos uma emoção ou nota.");
            return;
        }

        try {
            const existingEntry =
                settingsStore.getJournalEntryByDate(selectedDate);
            if (existingEntry) {
                await settingsStore.updateJournalEntry(existingEntry.id, {
                    content: journalNotes,
                    emotional_state_id: emotionalStateId || null,
                    intensity: emotionalIntensity,
                });
            } else {
                await settingsStore.addJournalEntry({
                    date: selectedDate,
                    content: journalNotes,
                    emotional_state_id: emotionalStateId || null,
                    intensity: emotionalIntensity,
                });
            }

            toast.success($t("psychology.checkin.success"));
            open = false;
            aiFeedback = ""; // Reset feedback
        } catch (e) {
            console.error("[DailyCheckinDialog] Save error:", e);
            toast.error("Erro ao salvar diário. Tente novamente.");
        }
    }

    async function analyzeEntry() {
        if (!journalNotes || journalNotes.length < 10) {
            toast.error("Escreva um pouco mais para a IA analisar.");
            return;
        }

        isAnalyzing = true;
        try {
            const emotionName =
                emotionalStates.find((e) => e.id === emotionalStateId)?.name ||
                "Não especificado";
            const feedback = await llmService.analyzeJournal(
                journalNotes,
                emotionName,
                emotionalIntensity,
            );
            aiFeedback = feedback;
            toast.success("Análise concluída!");
        } catch (e) {
            toast.error("Erro na análise: " + (e as Error).message);
        } finally {
            isAnalyzing = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2">
                <Brain class="w-5 h-5 text-primary" />
                {$t("psychology.checkin.title")}
            </Dialog.Title>
            <Dialog.Description>
                {$t("psychology.checkin.description")}
            </Dialog.Description>
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            <!-- Date -->
            <div class="space-y-2">
                <Label>{$t("psychology.checkin.date")}</Label>
                <Input type="date" bind:value={selectedDate} />
            </div>

            <!-- Emotion -->
            <div class="space-y-2">
                <Label>{$t("psychology.checkin.stateLabel")}</Label>
                <Select.Root type="single" bind:value={emotionalStateId}>
                    <Select.Trigger>
                        {emotionalStates.find((e) => e.id === emotionalStateId)
                            ?.name || $t("psychology.checkin.statePlaceholder")}
                    </Select.Trigger>
                    <Select.Content>
                        {#each emotionalStates as state}
                            <Select.Item value={state.id}
                                >{state.name}</Select.Item
                            >
                        {/each}
                    </Select.Content>
                </Select.Root>
            </div>

            <!-- Intensity -->
            <div class="space-y-2">
                <div class="flex justify-between">
                    <Label>{$t("psychology.checkin.intensityLabel")}</Label>
                    <span class="text-xs text-muted-foreground font-mono"
                        >{emotionalIntensity}/10</span
                    >
                </div>
                <Slider
                    bind:value={emotionalIntensity}
                    min={0}
                    max={10}
                    step={1}
                />
                <div class="flex justify-between text-xs text-muted-foreground">
                    <span>Leve</span>
                    <span>Intenso</span>
                </div>
            </div>

            <!-- Notes -->
            <div class="space-y-2">
                <div class="flex justify-between items-center">
                    <Label>{$t("psychology.checkin.notesLabel")}</Label>
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-6 text-xs text-blue-600 hover:text-blue-700 hover:bg-blue-50"
                        onclick={analyzeEntry}
                        disabled={isAnalyzing}
                    >
                        {#if isAnalyzing}
                            <Loader2 class="w-3 h-3 mr-1 animate-spin" />
                            Analisando...
                        {:else}
                            <Sparkles class="w-3 h-3 mr-1" />
                            Analisar com IA
                        {/if}
                    </Button>
                </div>
                <Textarea
                    placeholder={$t("psychology.checkin.notesPlaceholder")}
                    bind:value={journalNotes}
                    class="h-[120px]"
                />
            </div>

            <!-- AI Feedback Area -->
            {#if aiFeedback}
                <div
                    class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-3 animate-in fade-in slide-in-from-top-2"
                >
                    <div class="flex items-center gap-2 mb-2">
                        <MessageSquare class="w-4 h-4 text-blue-500" />
                        <span
                            class="text-xs font-bold text-blue-700 dark:text-blue-300"
                            >Feedback do Mentor IA</span
                        >
                    </div>
                    <p
                        class="text-xs text-muted-foreground whitespace-pre-line leading-relaxed"
                    >
                        {aiFeedback}
                    </p>
                </div>
            {/if}
        </div>

        <Dialog.Footer>
            <Button variant="outline" onclick={() => (open = false)}
                >{$t("general.cancel")}</Button
            >
            <Button onclick={save}>{$t("psychology.checkin.save")}</Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
