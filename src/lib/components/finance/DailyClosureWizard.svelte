<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Table from "$lib/components/ui/table";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { tradesStore } from "$lib/stores/trades.svelte";
    import { toast } from "svelte-sonner";
    import { Loader2, ArrowRight, CheckCircle2 } from "lucide-svelte";
    import { t, locale } from "svelte-i18n";
    import * as Select from "$lib/components/ui/select";
    import { Textarea } from "$lib/components/ui/textarea";
    import Slider from "$lib/components/ui/slider/slider.svelte";
    import { getLocalDatePart, formatLocalISO } from "$lib/utils";

    let { open = $bindable(false) } = $props();

    let step = $state(1);
    let selectedDate = $state(
        new Date(Date.now() - 86400000).toISOString().split("T")[0],
    ); // Defaults to yesterday
    let previewData = $state<any[]>([]);
    let selectedAccounts = $state<string[]>([]);
    let processing = $state(false);

    // Emotional State
    let emotionalStateId = $state<string>("");
    let emotionalIntensity = $state(5);
    let journalNotes = $state("");

    const emotionalStates = $derived(settingsStore.emotionalStates);

    function nextStep() {
        if (step === 1) {
            fetchPreview();
        } else if (step === 2) {
            // Check if journal entry exists for this date to pre-fill?
            const existingEntry =
                settingsStore.getJournalEntryByDate(selectedDate);
            if (existingEntry) {
                emotionalStateId = existingEntry.emotional_state_id || "";
                emotionalIntensity = existingEntry.intensity;
                journalNotes = existingEntry.content;
            } else {
                // Reset and Auto-Detect
                emotionalIntensity = 5;
                journalNotes = "";
                detectEmotionalState();
            }
            step = 3;
        } else if (step === 3) {
            confirmClosure();
        }
    }

    function fetchPreview() {
        // Fetch all data
        const allData = tradesStore.getDailyResultByAccount(
            selectedDate,
            settingsStore.accounts,
        );

        // Filter to only show accounts that HAVE trades or had a result
        previewData = allData.filter(
            (d) => d.trades_count > 0 || d.result !== 0,
        );

        selectedAccounts = previewData.map((d) => d.account_id); // Select all active by default
        step = 2;
    }

    function detectEmotionalState() {
        // Logic: Find trades for the selected date across accounts
        // and pick the most frequent entry/exit emotional state.
        const dayTrades = tradesStore.trades.filter(
            (t) => getLocalDatePart(t.date) === selectedDate,
        );
        if (dayTrades.length === 0) return;

        const counts: Record<string, number> = {};
        dayTrades.forEach((t) => {
            if (t.entry_emotional_state_id)
                counts[t.entry_emotional_state_id] =
                    (counts[t.entry_emotional_state_id] || 0) + 1;
            if (t.exit_emotional_state_id)
                counts[t.exit_emotional_state_id] =
                    (counts[t.exit_emotional_state_id] || 0) + 1;
        });

        const sorted = Object.entries(counts).sort((a, b) => b[1] - a[1]);
        if (sorted.length > 0) {
            emotionalStateId = sorted[0][0];
        }
    }

    async function confirmClosure() {
        processing = true;
        try {
            console.log(
                "[DailyClosureWizard] Starting closure process for date:",
                selectedDate,
            );
            // Simulate API delay
            await new Promise((r) => setTimeout(r, 1000));

            const accountsToProcess = previewData.filter((d) =>
                selectedAccounts.includes(d.account_id),
            );

            console.log(
                `[DailyClosureWizard] Processing ${accountsToProcess.length} accounts`,
            );

            for (const data of accountsToProcess) {
                if (data.result !== 0) {
                    // Find trades for this account and date to link them
                    // Using normalized date comparison for robustness
                    const linkedTrades = tradesStore.trades
                        .filter(
                            (t) =>
                                t.account_id === data.account_id &&
                                getLocalDatePart(t.date) === selectedDate,
                        )
                        .map((t) => t.id);

                    console.log(
                        `[DailyClosureWizard] Saving transaction for ${data.account_name}: ${data.result}. Linked trades: ${linkedTrades.length}`,
                    );

                    const txResult = await settingsStore.addCashTransaction({
                        id: `daily_closure_${data.account_id.replace(/:/g, "_")}_${selectedDate.replace(/-/g, "_")}`,
                        date: formatLocalISO(selectedDate),
                        amount: data.result,
                        type: data.result >= 0 ? "Deposit" : "Withdraw",
                        description: `${$t("finance.dailyClosure")} (${data.trades_count} ${$t("sidebar.trades").toLowerCase()})`,
                        account_id: data.account_id,
                        trade_ids: linkedTrades,
                        category: "Trading",
                        system_linked: true,
                    });

                    if (txResult && !txResult.success) {
                        throw new Error(
                            `Failed to save transaction for ${data.account_name}: ${txResult.error}`,
                        );
                    }
                }
            }

            // Save Journal Entry
            console.log(
                "[DailyClosureWizard] Transactions saved. Saving journal entry...",
            );
            if (emotionalStateId || journalNotes) {
                // Check existence again to decide update vs add
                const existingEntry =
                    settingsStore.getJournalEntryByDate(selectedDate);
                if (existingEntry) {
                    console.log(
                        "[DailyClosureWizard] Updating existing journal entry:",
                        existingEntry.id,
                    );
                    await settingsStore.updateJournalEntry(existingEntry.id, {
                        content: journalNotes,
                        emotional_state_id: emotionalStateId || null,
                        intensity: emotionalIntensity,
                    });
                } else {
                    console.log(
                        "[DailyClosureWizard] Creating new journal entry",
                    );
                    await settingsStore.addJournalEntry({
                        date: selectedDate,
                        content: journalNotes,
                        emotional_state_id: emotionalStateId || null,
                        intensity: emotionalIntensity,
                    });
                }
            }

            toast.success($t("finance.closureWizard.success"));
            step = 4;
        } catch (e: any) {
            console.error("[DailyClosureWizard] Closure Error:", e);
            toast.error(
                `${$t("finance.closureWizard.error")}: ${e.message || e}`,
            );
        } finally {
            processing = false;
        }
    }

    function close() {
        open = false;
        // Reset state after close animation
        setTimeout(() => {
            step = 1;
            selectedDate = new Date(Date.now() - 86400000)
                .toISOString()
                .split("T")[0];
        }, 300);
    }

    function toggleAccount(id: string) {
        if (selectedAccounts.includes(id)) {
            selectedAccounts = selectedAccounts.filter((a) => a !== id);
        } else {
            selectedAccounts = [...selectedAccounts, id];
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title>{$t("finance.closureWizard.title")}</Dialog.Title>
            <Dialog.Description>
                {$t("finance.closureWizard.description")}
            </Dialog.Description>
        </Dialog.Header>

        <div class="py-4">
            {#if step === 1}
                <div class="space-y-4">
                    <div class="space-y-2">
                        <Label>{$t("finance.closureWizard.step1.label")}</Label>
                        <Input type="date" bind:value={selectedDate} />
                        <p class="text-xs text-muted-foreground">
                            {$t("finance.closureWizard.step1.hint")}
                        </p>
                    </div>

                    {#if step === 1 && selectedDate}
                        {@const count = tradesStore.trades.filter(
                            (t) => t.date === selectedDate,
                        ).length}
                        {#if count === 0}
                            <div
                                class="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded text-yellow-500 text-sm"
                            >
                                {$t(
                                    "finance.closureWizard.step1.noTradesWarning",
                                )}
                            </div>
                        {/if}
                    {/if}
                </div>
            {:else if step === 2}
                <div class="space-y-4">
                    <div class="rounded-md border">
                        <Table.Root>
                            <Table.Header>
                                <Table.Row>
                                    <Table.Head class="w-[50px]">
                                        <Checkbox
                                            checked={selectedAccounts.length ===
                                                previewData.length &&
                                                previewData.length > 0}
                                            onCheckedChange={(v: boolean) =>
                                                (selectedAccounts = v
                                                    ? previewData.map(
                                                          (d) => d.account_id,
                                                      )
                                                    : [])}
                                        />
                                    </Table.Head>
                                    <Table.Head
                                        >{$t(
                                            "finance.closureWizard.step2.columns.account",
                                        )}</Table.Head
                                    >
                                    <Table.Head
                                        >{$t(
                                            "finance.closureWizard.step2.columns.trades",
                                        )}</Table.Head
                                    >
                                    <Table.Head class="text-right"
                                        >{$t(
                                            "finance.closureWizard.step2.columns.result",
                                        )}</Table.Head
                                    >
                                </Table.Row>
                            </Table.Header>
                            <Table.Body>
                                {#each previewData as item}
                                    <Table.Row>
                                        <Table.Cell>
                                            <Checkbox
                                                checked={selectedAccounts.includes(
                                                    item.account_id,
                                                )}
                                                onCheckedChange={() =>
                                                    toggleAccount(
                                                        item.account_id,
                                                    )}
                                            />
                                        </Table.Cell>
                                        <Table.Cell class="font-medium"
                                            >{item.account_name}</Table.Cell
                                        >
                                        <Table.Cell
                                            >{item.trades_count}</Table.Cell
                                        >
                                        <Table.Cell
                                            class="text-right font-mono font-bold {item.result >=
                                            0
                                                ? 'text-green-500'
                                                : 'text-red-500'}"
                                        >
                                            {item.result.toLocaleString(
                                                $locale || "pt-BR",
                                                {
                                                    style: "currency",
                                                    currency: "BRL",
                                                },
                                            )}
                                        </Table.Cell>
                                    </Table.Row>
                                {:else}
                                    <Table.Row>
                                        <Table.Cell
                                            colspan={4}
                                            class="text-center h-24 text-muted-foreground"
                                        >
                                            {$t(
                                                "finance.closureWizard.step2.noTrades",
                                            )}
                                        </Table.Cell>
                                    </Table.Row>
                                {/each}
                            </Table.Body>
                        </Table.Root>
                    </div>
                </div>
            {:else if step === 3}
                <!-- Emotional/Journal Step -->
                <div class="space-y-4">
                    <div class="flex flex-col space-y-2">
                        <Label
                            >{$t(
                                "finance.closureWizard.stepEmotion.title",
                            )}</Label
                        >
                        <p class="text-sm text-muted-foreground mb-4">
                            {$t(
                                "finance.closureWizard.stepEmotion.description",
                            )}
                        </p>
                    </div>

                    <div class="grid gap-4">
                        <div class="space-y-2">
                            <Label
                                >{$t(
                                    "finance.closureWizard.stepEmotion.stateLabel",
                                )}</Label
                            >
                            <Select.Root
                                type="single"
                                bind:value={emotionalStateId}
                            >
                                <Select.Trigger>
                                    {emotionalStates.find(
                                        (e) => e.id === emotionalStateId,
                                    )?.name ||
                                        $t(
                                            "finance.closureWizard.stepEmotion.statePlaceholder",
                                        )}
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

                        <div class="space-y-2">
                            <Label
                                >{$t(
                                    "finance.closureWizard.stepEmotion.intensityLabel",
                                    { values: { value: emotionalIntensity } },
                                )}</Label
                            >
                            <Slider
                                bind:value={emotionalIntensity}
                                min={0}
                                max={10}
                                step={1}
                            />
                            <div
                                class="flex justify-between text-xs text-muted-foreground"
                            >
                                <span
                                    >{$t(
                                        "finance.closureWizard.stepEmotion.intensityLight",
                                    )}</span
                                >
                                <span
                                    >{$t(
                                        "finance.closureWizard.stepEmotion.intensityIntense",
                                    )}</span
                                >
                            </div>
                        </div>

                        <div class="space-y-2">
                            <Label
                                >{$t(
                                    "finance.closureWizard.stepEmotion.notesLabel",
                                )}</Label
                            >
                            <Textarea
                                placeholder={$t(
                                    "finance.closureWizard.stepEmotion.notesPlaceholder",
                                )}
                                bind:value={journalNotes}
                                class="h-[100px]"
                            />
                        </div>
                    </div>
                </div>
            {:else if step === 4}
                <div
                    class="flex flex-col items-center justify-center py-8 text-center space-y-4"
                >
                    <div
                        class="w-16 h-16 rounded-full bg-green-100 flex items-center justify-center dark:bg-green-900/20"
                    >
                        <CheckCircle2
                            class="w-8 h-8 text-green-600 dark:text-green-400"
                        />
                    </div>
                    <div>
                        <h3 class="text-lg font-semibold">
                            {$t("finance.closureWizard.step3.successTitle")}
                        </h3>
                        <p class="text-muted-foreground">
                            {$t(
                                "finance.closureWizard.step3.successDescription",
                            )}
                        </p>
                    </div>
                </div>
            {/if}
        </div>

        <Dialog.Footer>
            {#if step === 1}
                <Button variant="outline" onclick={() => (open = false)}
                    >{$t("general.cancel")}</Button
                >
                <Button onclick={nextStep}>
                    {$t("general.next")}
                    <ArrowRight class="w-4 h-4 ml-2" />
                </Button>
            {:else if step === 2}
                <Button variant="outline" onclick={() => (step = 1)}
                    >{$t("general.back")}</Button
                >
                <Button onclick={nextStep}>
                    {$t("general.next")}
                    <ArrowRight class="w-4 h-4 ml-2" />
                </Button>
            {:else if step === 3}
                <Button variant="outline" onclick={() => (step = 2)}
                    >{$t("general.back")}</Button
                >
                <Button
                    onclick={nextStep}
                    disabled={selectedAccounts.length === 0 || processing}
                >
                    {#if processing}
                        <Loader2 class="w-4 h-4 mr-2 animate-spin" />
                        {$t("finance.closureWizard.step2.processing")}
                    {:else}
                        {$t("finance.closureWizard.step2.confirm")}
                    {/if}
                </Button>
            {:else if step === 4}
                <Button onclick={close}
                    >{$t("finance.closureWizard.step3.finish")}</Button
                >
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
