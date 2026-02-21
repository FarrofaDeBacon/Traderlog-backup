<script lang="ts">
    import { t } from "svelte-i18n";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Table from "$lib/components/ui/table";
    import { Plus, Trash2, TrendingUp, Info } from "lucide-svelte";
    import * as Tooltip from "$lib/components/ui/tooltip";

    let {
        partials = $bindable([]),
        entryPrice = 0,
        totalQuantity = 0,
        unitLabel = "contratos",
        resultSuffix = "pts",
        resultPrefix = "",
    } = $props<{
        partials: any[];
        entryPrice: number;
        totalQuantity: number;
        unitLabel?: string;
        resultSuffix?: string;
        resultPrefix?: string;
    }>();

    let nextType = $state("exit");

    function addPartial() {
        partials = [
            ...partials,
            {
                type: nextType,
                quantity: 1,
                price: entryPrice,
                date: new Date().toISOString().slice(0, 16),
                notes: "",
            },
        ];
    }

    function removePartial(index: number) {
        partials = partials.filter((_: any, i: number) => i !== index);
    }

    let totalExited = $derived(
        partials.reduce(
            (sum: number, p: any) =>
                p.type === "exit"
                    ? sum + (p.quantity || 0)
                    : sum - (p.quantity || 0),
            0,
        ),
    );

    let remaining = $derived(totalQuantity - totalExited);

    function calculateResult(partial: any) {
        if (!partial.price || !entryPrice || !partial.quantity) return 0;
        const diff = partial.price - entryPrice;
        // Direction multiplier logic should ideally come from parent,
        // but for display points, raw diff is usually expected.
        return diff * partial.quantity;
    }
</script>

<div class="space-y-4">
    <!-- Header/Summary Card -->
    <div
        class="flex items-center justify-between p-3 bg-muted/20 border border-muted rounded-lg"
    >
        <div class="flex gap-6">
            <div class="space-y-0.5">
                <p
                    class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground"
                >
                    {$t("trades.wizard.sections.partial_management.volume_executed")}
                </p>
                <p
                    class="text-sm font-mono font-bold text-primary flex items-center gap-2"
                ></p>
                <p
                    class="text-sm font-mono font-bold text-primary flex items-center gap-2"
                >
                    {totalExited} / {totalQuantity}
                    <span
                        class="text-[9px] text-muted-foreground font-normal lowercase"
                        >{unitLabel}</span
                    >
                </p>
            </div>
            <div class="space-y-0.5">
                <p
                    class="text-[9px] font-bold uppercase tracking-widest text-muted-foreground"
                >
                    {$t("trades.wizard.sections.partial_management.remaining")}
                </p>
                <p
                    class="text-sm font-mono font-bold {remaining < 0
                        ? 'text-red-500'
                        : 'text-zinc-400'}"
                >
                    {remaining}
                </p>
            </div>
        </div>

        <div class="flex items-center gap-2">
            <!-- Pre-seletor de Tipo (c4e6b7d1) -->
            <div
                class="flex items-center bg-black/40 rounded-lg p-0.5 border border-white/10 gap-0.5"
            >
                <button
                    onclick={() => (nextType = "exit")}
                    class="h-6 px-3 rounded-md text-[8px] font-black transition-all {nextType ===
                    'exit'
                        ? 'bg-blue-500 text-white shadow-lg shadow-blue-500/20'
                        : 'text-muted-foreground hover:text-white/50'}"
                >
                    {$t("trades.wizard.sections.partial_management.exit")}
                </button>
                <button
                    onclick={() => (nextType = "entry")}
                    class="h-6 px-3 rounded-md text-[8px] font-black transition-all {nextType ===
                    'entry'
                        ? 'bg-emerald-500 text-white shadow-lg shadow-emerald-500/20'
                        : 'text-muted-foreground hover:text-white/50'}"
                >
                    {$t("trades.wizard.sections.partial_management.addition")}
                </button>
            </div>

            <Button
                variant="ghost"
                size="sm"
                onclick={addPartial}
                class="h-8 text-[10px] font-bold uppercase tracking-wider bg-primary/10 text-primary hover:bg-primary/20"
            >
                <Plus class="w-3 h-3 mr-1.5" />
                {$t("trades.wizard.sections.partial_management.add_partial")}
            </Button>
        </div>
    </div>

    <!-- Partials Table -->
    <div class="rounded-lg border border-muted overflow-hidden bg-black/20">
        <Table.Root>
            <Table.Header>
                <Table.Row class="hover:bg-transparent border-muted/50">
                    <Table.Head
                        class="w-[100px] h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80"
                        >{$t("trades.wizard.sections.partial_management.type")}</Table.Head
                    >
                    <Table.Head
                        class="w-[160px] h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80"
                        >{$t("trades.wizard.sections.partial_management.date_time")}</Table.Head
                    >
                    <Table.Head
                        class="w-[110px] h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80 text-right"
                        >{$t("trades.wizard.sections.partial_management.price")}</Table.Head
                    >
                    <Table.Head
                        class="w-[80px] h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80 text-center"
                        >{$t("trades.wizard.sections.partial_management.quantity")}</Table.Head
                    >
                    <Table.Head
                        class="w-[120px] h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80 text-right"
                        >{$t("trades.wizard.sections.partial_management.result")}</Table.Head
                    >
                    <Table.Head
                        class="h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80"
                        >{$t("trades.wizard.sections.partial_management.observation")}</Table.Head
                    >
                    <Table.Head
                        class="w-[50px] h-8 text-[10px] font-bold uppercase tracking-tight text-muted-foreground/80 text-right"
                    ></Table.Head>
                </Table.Row>
            </Table.Header>
            <Table.Body>
                {#if partials.length === 0}
                    <Table.Row class="hover:bg-transparent border-0">
                        <Table.Cell colspan={7} class="h-24 text-center">
                            <div
                                class="flex flex-col items-center justify-center opacity-40 gap-2"
                            >
                                <TrendingUp class="w-6 h-6" />
                                <span
                                    class="text-[10px] uppercase font-bold tracking-widest"
                                    >{$t("trades.wizard.sections.partial_management.no_partials")}</span
                                >
                            </div>
                        </Table.Cell>
                    </Table.Row>
                {:else}
                    {#each partials as partial, i}
                        <Table.Row
                            class="hover:bg-muted/10 border-muted/30 group transition-colors"
                        >
                            <Table.Cell class="py-1">
                                <button
                                    onclick={() =>
                                        (partial.type =
                                            partial.type === "exit"
                                                ? "entry"
                                                : "exit")}
                                    class="px-2 py-0.5 rounded-[4px] text-[8px] font-black border transition-all {partial.type ===
                                    'exit'
                                        ? 'bg-blue-500/20 text-blue-400 border-blue-500/40 shadow-[0_0_10px_rgba(59,130,246,0.2)]'
                                        : 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40 shadow-[0_0_10px_rgba(16,185,129,0.2)]'}"
                                >
                                    {partial.type === "exit"
                                        ? $t("trades.wizard.sections.partial_management.exit")
                                        : $t("trades.wizard.sections.partial_management.addition")}
                                </button>
                            </Table.Cell>
                            <Table.Cell class="py-1">
                                <Input
                                    type="datetime-local"
                                    bind:value={partial.date}
                                    class="h-7 w-full bg-transparent border-0 text-[10px] font-mono p-0 focus-visible:ring-0 text-muted-foreground focus:text-white transition-colors"
                                />
                            </Table.Cell>
                            <Table.Cell class="py-1">
                                <Input
                                    type="number"
                                    step="0.01"
                                    bind:value={partial.price}
                                    class="h-7 w-full bg-transparent border-0 text-[10px] font-mono text-right p-0 focus-visible:ring-0 text-white font-bold"
                                />
                            </Table.Cell>
                            <Table.Cell class="py-1">
                                <Input
                                    type="number"
                                    bind:value={partial.quantity}
                                    class="h-7 w-full bg-transparent border-0 text-[10px] font-mono text-center p-0 focus-visible:ring-0 text-white"
                                />
                            </Table.Cell>
                            <Table.Cell class="py-1 text-right">
                                <div
                                    class="text-[10px] font-mono font-bold {calculateResult(
                                        partial,
                                    ) > 0
                                        ? 'text-emerald-500'
                                        : calculateResult(partial) < 0
                                          ? 'text-red-500'
                                          : 'text-muted-foreground'}"
                                >
                                    {resultPrefix}
                                    {calculateResult(partial).toFixed(2)}
                                    <span class="text-[8px] opacity-50"
                                        >{resultSuffix}</span
                                    >
                                </div>
                            </Table.Cell>
                            <Table.Cell class="py-1">
                                <Input
                                    type="text"
                                    placeholder="-"
                                    bind:value={partial.notes}
                                    class="h-7 w-full bg-transparent border-0 text-[10px] p-0 focus-visible:ring-0 placeholder:text-muted-foreground/30 focus:placeholder:text-muted-foreground/50"
                                />
                            </Table.Cell>
                            <Table.Cell class="py-1 text-right">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onclick={() => removePartial(i)}
                                    class="h-6 w-6 text-muted-foreground hover:text-red-500 hover:bg-red-500/10 opacity-0 group-hover:opacity-100 transition-all"
                                >
                                    <Trash2 class="w-3 h-3" />
                                </Button>
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                {/if}
            </Table.Body>
        </Table.Root>
    </div>
</div>

<style>
    /* Remover setas do input number */
    :global(input[type="number"]::-webkit-inner-spin-button),
    :global(input[type="number"]::-webkit-outer-spin-button) {
        -webkit-appearance: none;
        margin: 0;
    }
</style>
