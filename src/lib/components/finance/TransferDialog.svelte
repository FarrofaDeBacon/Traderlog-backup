<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Select from "$lib/components/ui/select";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Button } from "$lib/components/ui/button";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import {
        ArrowRightLeft,
        Wallet,
        ChevronRight,
        Info,
        AlertCircle,
        ArrowDown,
    } from "lucide-svelte";
    import { t } from "svelte-i18n";
    import { toast } from "svelte-sonner";
    import { cn } from "$lib/utils";

    let { open = $bindable(false) } = $props();

    let fromAccountId = $state("");
    let toAccountId = $state("");
    let sourceAmount = $state(0);
    let fee = $state(0);
    let destAmount = $state(0);
    let exchangeRate = $state(1);
    let date = $state(new Date().toISOString().split("T")[0]);
    let description = $state("");

    let fromAccount = $derived(
        settingsStore.accounts.find((a) => a.id === fromAccountId),
    );
    let toAccount = $derived(
        settingsStore.accounts.find((a) => a.id === toAccountId),
    );

    let sameCurrency = $derived(
        fromAccount && toAccount && fromAccount.currency === toAccount.currency,
    );

    // Update exchange rate when accounts change
    $effect(() => {
        if (fromAccount && toAccount && !sameCurrency) {
            const fromRate =
                settingsStore.currencies.find(
                    (c) => c.code === fromAccount!.currency,
                )?.exchange_rate || 1;
            const toRate =
                settingsStore.currencies.find(
                    (c) => c.code === toAccount!.currency,
                )?.exchange_rate || 1;
            exchangeRate = fromRate / toRate;
            handleSourceChange();
        } else {
            exchangeRate = 1;
            handleSourceChange();
        }
    });

    function handleSourceChange() {
        destAmount = (sourceAmount - fee) * exchangeRate;
    }

    function handleDestChange() {
        if (exchangeRate > 0) {
            sourceAmount = destAmount / exchangeRate + fee;
        }
    }

    async function handleSubmit() {
        if (!fromAccountId || !toAccountId || fromAccountId === toAccountId) {
            toast.error("Selecione contas de origem e destino diferentes.");
            return;
        }

        if (sourceAmount <= 0) {
            toast.error("O valor deve ser maior que zero.");
            return;
        }

        const result = await settingsStore.transferFunds({
            fromAccountId,
            toAccountId,
            amountParams: {
                sourceAmount: sourceAmount,
                fee: fee,
                destAmount: destAmount,
            },
            date,
            description:
                description ||
                `Transf: ${fromAccount?.currency} -> ${toAccount?.currency} (Rate: ${exchangeRate.toFixed(4)})`,
        });

        if (result.success) {
            toast.success("Transferência realizada com sucesso!");
            open = false;
            resetForm();
        } else {
            toast.error(result.error || "Erro ao realizar transferência.");
        }
    }

    function resetForm() {
        fromAccountId = "";
        toAccountId = "";
        sourceAmount = 0;
        fee = 0;
        destAmount = 0;
        description = "";
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content
        class="sm:max-w-[480px] bg-zinc-900 border-zinc-800 p-0 overflow-hidden"
    >
        <div
            class="p-6 border-b border-zinc-800 bg-zinc-900 shadow-xl relative z-10"
        >
            <Dialog.Header>
                <Dialog.Title
                    class="text-xl font-bold text-white flex items-center gap-2"
                >
                    <div
                        class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center"
                    >
                        <ArrowRightLeft class="w-5 h-5 text-primary" />
                    </div>
                    Nova Transferência
                </Dialog.Title>
                <Dialog.Description class="text-zinc-500 text-xs">
                    Transfira fundos entre suas contas com conversão automática.
                </Dialog.Description>
            </Dialog.Header>
        </div>

        <div
            class="p-6 space-y-6 max-h-[70vh] overflow-y-auto custom-scrollbar"
        >
            <!-- Account Selection -->
            <div
                class="grid grid-cols-[1fr,auto,1fr] items-center gap-2 bg-zinc-950/50 p-4 rounded-2xl border border-zinc-800/50"
            >
                <div class="space-y-2">
                    <Label
                        class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1"
                        >Origem</Label
                    >
                    <Select.Root type="single" bind:value={fromAccountId}>
                        <Select.Trigger
                            class="w-full bg-zinc-900 border-zinc-800 h-10"
                        >
                            <span class="truncate">
                                {fromAccount?.nickname ?? "Origem"}
                            </span>
                        </Select.Trigger>
                        <Select.Content class="bg-zinc-900 border-zinc-800">
                            {#each settingsStore.accounts as acc}
                                <Select.Item
                                    value={acc.id}
                                    class="text-zinc-300 focus:bg-zinc-800"
                                    >{acc.nickname} ({acc.currency})</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>

                <div class="mt-6">
                    <div
                        class="w-8 h-8 rounded-full bg-zinc-800 flex items-center justify-center"
                    >
                        <ChevronRight class="w-4 h-4 text-zinc-500" />
                    </div>
                </div>

                <div class="space-y-2">
                    <Label
                        class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1 text-right block"
                        >Destino</Label
                    >
                    <Select.Root type="single" bind:value={toAccountId}>
                        <Select.Trigger
                            class="w-full bg-zinc-900 border-zinc-800 h-10"
                        >
                            <span class="truncate">
                                {toAccount?.nickname ?? "Destino"}
                            </span>
                        </Select.Trigger>
                        <Select.Content class="bg-zinc-900 border-zinc-800">
                            {#each settingsStore.accounts as acc}
                                <Select.Item
                                    value={acc.id}
                                    class="text-zinc-300 focus:bg-zinc-800"
                                    >{acc.nickname} ({acc.currency})</Select.Item
                                >
                            {/each}
                        </Select.Content>
                    </Select.Root>
                </div>
            </div>

            <!-- Amounts -->
            <div class="space-y-4">
                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-2">
                        <Label
                            class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1"
                            >Valor Total na Origem</Label
                        >
                        <div class="relative">
                            <div
                                class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 font-bold text-xs"
                            >
                                {fromAccount?.currency ?? ""}
                            </div>
                            <Input
                                type="number"
                                step="0.01"
                                class="pl-12 h-12 bg-zinc-950/50 border-zinc-800 font-mono font-bold"
                                bind:value={sourceAmount}
                                oninput={handleSourceChange}
                            />
                        </div>
                    </div>
                    <div class="space-y-2">
                        <Label
                            class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1"
                            >Taxas / Custos</Label
                        >
                        <div class="relative">
                            <div
                                class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 font-bold text-xs"
                            >
                                {fromAccount?.currency ?? ""}
                            </div>
                            <Input
                                type="number"
                                step="0.01"
                                class="pl-12 h-12 bg-zinc-950/50 border-zinc-800 font-mono"
                                bind:value={fee}
                                oninput={handleSourceChange}
                            />
                        </div>
                    </div>
                </div>

                {#if fromAccount && toAccount && !sameCurrency}
                    <div
                        class="bg-primary/5 border border-primary/20 rounded-xl p-4 space-y-3"
                    >
                        <div class="flex items-center justify-between">
                            <div
                                class="flex items-center gap-2 text-primary font-bold text-[10px] uppercase tracking-wider"
                            >
                                <Info class="w-3 h-3" />
                                Taxa de Câmbio
                            </div>
                            <div class="text-[10px] text-zinc-500 font-mono">
                                1 {fromAccount.currency} = {exchangeRate.toFixed(
                                    4,
                                )}
                                {toAccount.currency}
                            </div>
                        </div>
                        <Input
                            type="number"
                            step="0.0001"
                            class="bg-zinc-950/50 border-primary/20 h-10 font-mono text-center text-primary font-bold"
                            bind:value={exchangeRate}
                            oninput={handleSourceChange}
                        />
                    </div>
                {/if}

                <div class="flex justify-center py-2">
                    <div
                        class="w-8 h-8 rounded-full bg-zinc-800 flex items-center justify-center"
                    >
                        <ArrowDown class="w-4 h-4 text-zinc-500" />
                    </div>
                </div>

                <div class="space-y-2">
                    <Label
                        class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1"
                        >Valor Final Creditado</Label
                    >
                    <div class="relative">
                        <div
                            class="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 font-bold text-xs uppercase"
                        >
                            {toAccount?.currency ?? ""}
                        </div>
                        <Input
                            type="number"
                            step="0.01"
                            class="pl-12 h-14 bg-zinc-950/80 border-emerald-500/30 text-emerald-400 font-mono text-xl font-black"
                            bind:value={destAmount}
                            oninput={handleDestChange}
                        />
                    </div>
                    <p class="text-[10px] text-zinc-600 italic px-1">
                        Valor após taxas e conversão cambial
                    </p>
                </div>
            </div>

            <!-- Other Info -->
            <div class="grid grid-cols-2 gap-4">
                <div class="space-y-2">
                    <Label
                        class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1"
                        >Data</Label
                    >
                    <Input
                        type="date"
                        class="bg-zinc-950/50 border-zinc-800 h-10"
                        bind:value={date}
                    />
                </div>
                <div class="space-y-2">
                    <Label
                        class="text-[10px] font-bold text-zinc-500 uppercase tracking-widest px-1"
                        >Descrição</Label
                    >
                    <Input
                        placeholder="Opcional"
                        class="bg-zinc-950/50 border-zinc-800 h-10"
                        bind:value={description}
                    />
                </div>
            </div>

            {#if fromAccountId === toAccountId && fromAccountId !== ""}
                <div
                    class="bg-red-500/10 border border-red-500/20 rounded-lg p-3 flex items-center gap-3 text-red-500 text-xs"
                >
                    <AlertCircle class="w-4 h-4 shrink-0" />
                    As contas de origem e destino devem ser diferentes.
                </div>
            {/if}
        </div>

        <Dialog.Footer class="p-6 bg-zinc-900 border-t border-zinc-800">
            <Button
                variant="ghost"
                onclick={() => (open = false)}
                class="text-zinc-500">Cancelar</Button
            >
            <Button
                class="px-8 shadow-lg shadow-primary/20"
                onclick={handleSubmit}
                disabled={!fromAccountId ||
                    !toAccountId ||
                    fromAccountId === toAccountId ||
                    sourceAmount <= 0}
            >
                Confirmar Transferência
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #27272a;
        border-radius: 10px;
    }
</style>
