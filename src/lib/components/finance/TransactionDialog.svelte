<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Select from "$lib/components/ui/select";
    import * as Tabs from "$lib/components/ui/tabs";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";
    import {
        ArrowRight,
        ArrowRightLeft,
        Wallet,
        ArrowUpRight,
        ArrowDownLeft,
    } from "lucide-svelte";

    let { open = $bindable(false) } = $props();

    // UI State
    let activeTab = $state("transaction");

    // Shared Fields
    let date = $state(new Date().toISOString().split("T")[0]);
    let description = $state("");

    // Transaction Fields
    let transactionType = $state("Deposit");
    let accountId = $state("");
    let amount = $state("");

    // Transfer Fields
    let fromAccountId = $state("");
    let toAccountId = $state("");
    let transferAmount = $state("");
    let feePercent = $state("");
    let destAmount = $state("");
    let exchangeRate = $state("1.0");

    let accountOptions = $derived(
        settingsStore.accounts.map((a) => ({
            value: a.id,
            label: a.nickname,
            currency: a.currency,
        })),
    );

    let transactionTypes = $derived([
        { value: "Deposit", label: $t("finance.statement.types.deposit") },
        { value: "Withdraw", label: $t("finance.statement.types.withdraw") },
        {
            value: "Adjustment",
            label: $t("finance.statement.types.adjustment"),
        },
    ]);

    // Derived helpers for Transfer
    let fromAccount = $derived(
        settingsStore.accounts.find((a) => a.id === fromAccountId),
    );
    let toAccount = $derived(
        settingsStore.accounts.find((a) => a.id === toAccountId),
    );
    let sameCurrency = $derived(
        fromAccount && toAccount && fromAccount.currency === toAccount.currency,
    );

    let feeAmount = $derived.by(() => {
        const s = parseFloat(transferAmount) || 0;
        const p = parseFloat(feePercent) || 0;
        return (s * p) / 100;
    });

    // Transfer Logic
    function handleTransferSourceChange() {
        if (!transferAmount) return;
        const s = parseFloat(transferAmount) || 0;
        const rate = parseFloat(exchangeRate) || 1;
        const p = parseFloat(feePercent) || 0;
        const calculatedFee = (s * p) / 100;
        const net = Math.max(0, s - calculatedFee);

        if (sameCurrency) {
            destAmount = net.toFixed(2);
            exchangeRate = "1.0";
        } else {
            destAmount = (net * rate).toFixed(2);
        }
    }

    function handleTransferDestChange() {
        if (!destAmount || sameCurrency) return;
        const d = parseFloat(destAmount) || 0;
        const s = parseFloat(transferAmount) || 0;
        const p = parseFloat(feePercent) || 0;
        const calculatedFee = (s * p) / 100;
        const net = Math.max(0, s - calculatedFee);

        if (net > 0) {
            exchangeRate = (d / net).toFixed(4);
        }
    }

    async function save() {
        if (activeTab === "transfer") {
            if (
                !fromAccountId ||
                !toAccountId ||
                !transferAmount ||
                parseFloat(transferAmount) <= 0
            ) {
                toast.error("Preencha os campos obrigatórios.");
                return;
            }
            if (fromAccountId === toAccountId) {
                toast.error("Contas devem ser diferentes.");
                return;
            }

            const result = await settingsStore.transferFunds({
                fromAccountId,
                toAccountId,
                amountParams: {
                    sourceAmount: parseFloat(transferAmount),
                    fee: feeAmount,
                    destAmount: parseFloat(destAmount) || 0,
                },
                date,
                description:
                    description ||
                    (sameCurrency
                        ? "Transferência"
                        : `FX Rate: ${exchangeRate}`),
            });

            if (result.success) {
                toast.success("Transferência realizada com sucesso!");
                open = false;
                resetForms();
            } else {
                toast.error(result.error || "Erro ao realizar transferência.");
            }
        } else {
            // Standard Transaction
            if (!accountId || !amount || parseFloat(amount) <= 0) {
                toast.error("Preencha os campos obrigatórios.");
                return;
            }

            const value = parseFloat(amount);
            const finalAmount = transactionType === "Withdraw" ? -value : value;

            const result = await settingsStore.addCashTransaction({
                account_id: accountId,
                amount: finalAmount,
                date: date,
                type: transactionType as any,
                description:
                    description ||
                    (transactionType === "Deposit"
                        ? "Aporte Manual"
                        : transactionType === "Withdraw"
                          ? "Retirada Manual"
                          : "Ajuste"),
            });

            if (result.success) {
                toast.success("Operação realizada com sucesso!");
                open = false;
                resetForms();
            } else {
                toast.error(result.error || "Erro ao salvar transação.");
            }
        }
    }

    function resetForms() {
        amount = "";
        description = "";
        transferAmount = "";
        feePercent = "";
        destAmount = "";
        // Keep accounts/types for UX convenience
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title>Nova Operação</Dialog.Title>
            <Dialog.Description>
                Registre aportes, retiradas ou transferências entre contas.
            </Dialog.Description>
        </Dialog.Header>

        <Tabs.Root bind:value={activeTab} class="w-full">
            <Tabs.List class="grid w-full grid-cols-2">
                <Tabs.Trigger value="transaction" class="gap-2">
                    <Wallet class="w-4 h-4" /> Transação
                </Tabs.Trigger>
                <Tabs.Trigger value="transfer" class="gap-2">
                    <ArrowRightLeft class="w-4 h-4" /> Transferência
                </Tabs.Trigger>
            </Tabs.List>

            <!-- AB: TRANSACAO -->
            <Tabs.Content value="transaction" class="py-4 space-y-4">
                <div class="grid grid-cols-12 gap-4">
                    <div class="col-span-3 space-y-2">
                        <Label>{$t("finance.transactionDialog.type")}</Label>
                        <Select.Root type="single" bind:value={transactionType}>
                            <Select.Trigger class="w-full">
                                {transactionTypes.find(
                                    (o) => o.value === transactionType,
                                )?.label ?? transactionType}
                            </Select.Trigger>
                            <Select.Content>
                                {#each transactionTypes as option}
                                    <Select.Item value={option.value}
                                        >{option.label}</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>

                    <div class="col-span-5 space-y-2">
                        <Label>{$t("finance.transactionDialog.account")}</Label>
                        <Select.Root type="single" bind:value={accountId}>
                            <Select.Trigger class="w-full">
                                {accountOptions.find(
                                    (o) => o.value === accountId,
                                )?.label ?? $t("finance.statement.account")}
                            </Select.Trigger>
                            <Select.Content>
                                {#each accountOptions as acc}
                                    <Select.Item value={acc.value}
                                        >{acc.label} ({acc.currency})</Select.Item
                                    >
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>

                    <div class="col-span-4 space-y-2">
                        <Label>{$t("finance.transactionDialog.date")}</Label>
                        <Input type="date" bind:value={date} />
                    </div>
                </div>

                <div class="grid grid-cols-12 gap-4">
                    <div class="col-span-4 space-y-2">
                        <Label>{$t("finance.transactionDialog.amount")}</Label>
                        <div class="relative">
                            <Input
                                type="number"
                                step="0.01"
                                bind:value={amount}
                                class="pl-8"
                                placeholder="0.00"
                            />
                            <div
                                class="absolute inset-y-0 left-0 flex items-center pl-2.5 pointer-events-none text-muted-foreground"
                            >
                                {#if transactionType === "Withdraw"}
                                    <ArrowUpRight
                                        class="w-4 h-4 text-red-500"
                                    />
                                {:else}
                                    <ArrowDownLeft
                                        class="w-4 h-4 text-green-500"
                                    />
                                {/if}
                            </div>
                        </div>
                    </div>

                    <div class="col-span-8 space-y-2">
                        <Label
                            >{$t(
                                "finance.transactionDialog.description",
                            )}</Label
                        >
                        <Input
                            placeholder={$t(
                                "finance.transactionDialog.descriptionPlaceholder",
                            )}
                            bind:value={description}
                        />
                    </div>
                </div>
            </Tabs.Content>

            <!-- TAB: TRANSFERENCIA -->
            <Tabs.Content value="transfer" class="py-4 space-y-4">
                <div
                    class="flex items-center gap-4 bg-muted/30 p-3 rounded-lg border"
                >
                    <div class="flex-1 space-y-1">
                        <Label class="text-xs">De</Label>
                        <Select.Root type="single" bind:value={fromAccountId}>
                            <Select.Trigger class="bg-background h-9 w-full">
                                {accountOptions.find(
                                    (a) => a.value === fromAccountId,
                                )?.label ?? "Origem"}
                            </Select.Trigger>
                            <Select.Content>
                                {#each accountOptions as acc}
                                    <Select.Item
                                        value={acc.value}
                                        disabled={acc.value === toAccountId}
                                    >
                                        {acc.label} ({acc.currency})
                                    </Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>

                    <ArrowRight class="w-4 h-4 text-muted-foreground mt-5" />

                    <div class="flex-1 space-y-1">
                        <Label class="text-xs">Para</Label>
                        <Select.Root type="single" bind:value={toAccountId}>
                            <Select.Trigger class="bg-background h-9 w-full">
                                {accountOptions.find(
                                    (a) => a.value === toAccountId,
                                )?.label ?? "Destino"}
                            </Select.Trigger>
                            <Select.Content>
                                {#each accountOptions as acc}
                                    <Select.Item
                                        value={acc.value}
                                        disabled={acc.value === fromAccountId}
                                    >
                                        {acc.label} ({acc.currency})
                                    </Select.Item>
                                {/each}
                            </Select.Content>
                        </Select.Root>
                    </div>
                </div>

                <div class="grid grid-cols-12 gap-3">
                    <div class="col-span-3 space-y-1">
                        <Label class="text-xs"
                            >Valor ({fromAccount?.currency ?? ""})</Label
                        >
                        <Input
                            type="number"
                            step="0.01"
                            bind:value={transferAmount}
                            oninput={handleTransferSourceChange}
                            placeholder="0.00"
                        />
                    </div>

                    <div class="col-span-2 space-y-1">
                        <Label class="text-xs">Taxa (%)</Label>
                        <div class="relative">
                            <Input
                                type="number"
                                step="0.01"
                                bind:value={feePercent}
                                oninput={handleTransferSourceChange}
                                placeholder="0"
                                class="text-red-500 pr-0 text-center"
                            />
                        </div>
                    </div>

                    <div class="col-span-3 space-y-1">
                        <Label class="text-xs"
                            >Entrada ({toAccount?.currency ?? ""})</Label
                        >
                        <Input
                            type="number"
                            step="0.01"
                            bind:value={destAmount}
                            readonly={sameCurrency}
                            oninput={handleTransferDestChange}
                            placeholder="0.00"
                            class={sameCurrency
                                ? "bg-muted"
                                : "font-bold text-green-600"}
                        />
                    </div>

                    <div class="col-span-4 space-y-1">
                        <Label class="text-xs">Data</Label>
                        <Input type="date" bind:value={date} />
                    </div>
                </div>

                {#if !sameCurrency && fromAccountId && toAccountId}
                    <div
                        class="flex items-center gap-2 px-2 py-1 bg-yellow-500/10 border border-yellow-500/20 rounded text-xs text-yellow-600 dark:text-yellow-400"
                    >
                        <ArrowRightLeft class="w-3 h-3" />
                        <span>1 {fromAccount?.currency} = </span>
                        <Input
                            class="w-20 h-6 text-xs bg-transparent border-yellow-500/30 p-1"
                            type="number"
                            step="0.0001"
                            bind:value={exchangeRate}
                            oninput={handleTransferSourceChange}
                        />
                        <span>{toAccount?.currency}</span>
                    </div>
                {/if}

                <div class="space-y-1">
                    <Label class="text-xs">Detalhes</Label>
                    <Input
                        class="h-9"
                        placeholder="Opcional: Descrição da transferência..."
                        bind:value={description}
                    />
                </div>
            </Tabs.Content>
        </Tabs.Root>

        <Dialog.Footer>
            <Button variant="outline" onclick={() => (open = false)}
                >{$t("general.cancel")}</Button
            >
            <Button onclick={save}
                >{$t("finance.transactionDialog.confirm")}</Button
            >
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
```
