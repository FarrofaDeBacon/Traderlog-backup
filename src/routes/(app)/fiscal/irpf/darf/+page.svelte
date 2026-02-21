<script lang="ts">
    import { onMount } from "svelte";
    import { toast } from "svelte-sonner";
    import * as Card from "$lib/components/ui/card";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Dialog from "$lib/components/ui/dialog";
    import {
        FileText,
        CheckCircle,
        DollarSign,
        ArrowLeft,
        Trash2,
        Eye,
        AlertTriangle,
        Undo2,
    } from "lucide-svelte";
    import { irpfStore } from "$lib/stores/irpfStore.svelte";
    import { settingsStore } from "$lib/stores/settings.svelte"; // Import settingsStore
    import * as Select from "$lib/components/ui/select";
    let year = $state(new Date().getFullYear());

    // View Modal State
    let isViewModalOpen = $state(false);
    let selectedDarf = $state<any>(null);

    // Delete Modal State
    let isDeleteModalOpen = $state(false);
    let darfToDelete = $state<any>(null);

    // Payment Modal State
    let isPayModalOpen = $state(false);
    let paymentData = $state({
        id: "",
        principal: 0,
        fine: 0,
        interest: 0,
        total: 0,
        date: new Date().toISOString().split("T")[0],
        accountId: "", // Add accountId
    });

    onMount(() => {
        irpfStore.loadDarfs(year);
        // Ensure accounts are loaded if not already
        if (settingsStore.accounts.length === 0) {
            settingsStore.loadData();
        }
    });

    function loadData() {
        irpfStore.loadDarfs(year);
    }

    function openPayModal(item: any) {
        paymentData = {
            id: item.id,
            principal: item.principal_value,
            fine: item.fine || 0,
            interest: item.interest || 0,
            total: item.total_value || item.principal_value,
            date: new Date().toISOString().split("T")[0],
            accountId: "", // Reset accountId
        };
        isPayModalOpen = true;
    }

    function updatePaymentTotal() {
        paymentData.total =
            Number(paymentData.principal) +
            Number(paymentData.fine) +
            Number(paymentData.interest);
    }

    async function confirmPayment() {
        if (!paymentData.accountId) {
            toast.error("Selecione uma conta de origem.");
            return;
        }
        try {
            const idStr = irpfStore.getId(paymentData.id);
            await irpfStore.markDarfPaid(
                idStr,
                paymentData.date,
                paymentData.total,
                paymentData.accountId, // Pass accountId
                paymentData.fine,
                paymentData.interest,
            );
            isPayModalOpen = false;
        } catch (error) {
            // Handled in store
        }
    }

    function deleteDarf(item: any) {
        darfToDelete = item;
        isDeleteModalOpen = true;
    }

    async function confirmDelete() {
        if (!darfToDelete) return;
        const idStr = irpfStore.getId(darfToDelete.id);
        await irpfStore.deleteDarf(idStr);
        isDeleteModalOpen = false;
    }

    function openViewModal(darf: any) {
        selectedDarf = darf;
        isViewModalOpen = true;
    }

    function formatCurrency(val: number) {
        return new Intl.NumberFormat("pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(val || 0);
    }

    function formatDate(dateStr: string) {
        if (!dateStr) return "-";
        const [y, m, d] = dateStr.split("-");
        return `${d}/${m}/${y}`;
    }

    function getStatusColor(status: string, dueDate: string) {
        if (status === "Paid")
            return "text-green-500 bg-green-500/10 border-green-500/20";

        const today = new Date().toISOString().split("T")[0];
        if (dueDate < today)
            return "text-red-500 bg-red-500/10 border-red-500/20";

        return "text-yellow-500 bg-yellow-500/10 border-yellow-500/20";
    }

    // Unpay Modal State
    let isUnpayModalOpen = $state(false);
    let darfToUnpay = $state<any>(null);

    function openUnpayModal(item: any) {
        darfToUnpay = item;
        isUnpayModalOpen = true;
    }

    async function confirmUnpay() {
        if (!darfToUnpay) return;
        const idStr = irpfStore.getId(darfToUnpay.id);
        await irpfStore.unpayDarf(idStr);
        isUnpayModalOpen = false;
        isViewModalOpen = false; // Close view modal if open
    }

    function getAccountName(accountId: string) {
        if (!accountId) return "Desconhecida";
        const id = accountId.includes(":")
            ? accountId.split(":").pop()
            : accountId;
        const acc = settingsStore.accounts.find(
            (a) => irpfStore.getId(a.id) === id,
        );
        return acc ? acc.nickname : "Conta Removida";
    }

    let pendingDarfs = $derived(
        irpfStore.darfs
            .filter((d) => d.status === "Pending" && d.total_value > 0)
            .sort(
                (a, b) =>
                    new Date(a.due_date).getTime() -
                    new Date(b.due_date).getTime(),
            ),
    );

    let historyDarfs = $derived(
        irpfStore.darfs
            .filter((d) => {
                if (d.status === "Pending" && d.total_value > 0) return false; // Already in pending
                // Filter by year strictly for history (Parse MM/YYYY manually)
                const parts = d.period.split("/");
                const darfYear = parts.length > 1 ? parseInt(parts[1]) : 0;
                return darfYear === year;
            })
            .sort((a, b) => {
                // Sort by period descending (MM/YYYY)
                const partsA = a.period.split("/");
                const partsB = b.period.split("/");
                const dateA = new Date(
                    parseInt(partsA[1]),
                    parseInt(partsA[0]) - 1,
                );
                const dateB = new Date(
                    parseInt(partsB[1]),
                    parseInt(partsB[0]) - 1,
                );
                return dateB.getTime() - dateA.getTime();
            }),
    );

    let selectedYearOption = $derived({ value: year, label: year.toString() });
    let selectedYear = $state(year);
    function loadIrpfData() {
        year = selectedYear;
        loadData();
    }
</script>

<div
    class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20"
>
    <!-- Header & Actions -->
    <div
        class="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-white/10 pb-6"
    >
        <div class="flex items-center gap-4">
            <Button variant="ghost" size="icon" href="/fiscal/irpf">
                <ArrowLeft class="w-5 h-5" />
            </Button>
            <div>
                <h2 class="text-2xl font-bold text-white tracking-tight">
                    Gerenciamento de DARFs
                </h2>
                <p class="text-muted-foreground">
                    Central de pagamentos e histórico fiscal.
                </p>
            </div>
        </div>
        <div class="flex items-center gap-2">
            <Select.Root
                type="single"
                value={selectedYear.toString()}
                onValueChange={(v) => {
                    if (v) {
                        selectedYear = parseInt(v);
                        loadIrpfData();
                    }
                }}
            >
                <Select.Trigger
                    class="w-[140px] bg-black/20 border-white/10 text-white"
                >
                    {selectedYear}
                </Select.Trigger>
                <Select.Content>
                    {#each Array.from({ length: 5 }, (_, i) => new Date().getFullYear() - i) as y}
                        <Select.Item value={y.toString()} label={y.toString()}
                            >{y}</Select.Item
                        >
                    {/each}
                </Select.Content>
            </Select.Root>
            <Button variant="outline" onclick={loadData}>
                <Undo2 class="w-4 h-4 mr-2" /> Atualizar
            </Button>
        </div>
    </div>

    <!-- Active DARFs (Cards) -->
    {#if pendingDarfs.length > 0}
        <div>
            <h3
                class="text-lg font-semibold text-white mb-4 flex items-center gap-2"
            >
                <AlertTriangle class="w-5 h-5 text-amber-500" />
                Guias em Aberto
            </h3>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {#each pendingDarfs as item}
                    <Card.Root
                        class="bg-black/40 border-l-4 border-l-amber-500 border-y border-r border-white/10 glass relative overflow-hidden group hover:border-r-amber-500/30 transition-all"
                    >
                        <div class="absolute top-0 right-0 p-4 flex gap-2">
                            {#if item.due_date < new Date()
                                    .toISOString()
                                    .split("T")[0]}
                                <span
                                    class="px-2 py-1 rounded text-xs font-bold text-red-500 bg-red-500/10 border border-red-500/20"
                                >
                                    Atrasado
                                </span>
                            {:else}
                                <span
                                    class="px-2 py-1 rounded text-xs font-bold text-amber-500 bg-amber-500/10 border border-amber-500/20"
                                >
                                    Pendente
                                </span>
                            {/if}
                        </div>

                        <Card.Header>
                            <Card.Title class="flex items-center gap-2">
                                <FileText class="w-5 h-5 text-primary" />
                                DARF {item.period}
                            </Card.Title>
                            <Card.Description
                                >Cód: {item.revenue_code}</Card.Description
                            >
                        </Card.Header>

                        <Card.Content class="space-y-4">
                            <div class="space-y-1">
                                <span
                                    class="text-xs text-muted-foreground uppercase"
                                    >Valor a Pagar</span
                                >
                                <div
                                    class="text-2xl font-bold font-mono text-white"
                                >
                                    {formatCurrency(item.total_value)}
                                </div>
                            </div>

                            <div class="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                    <span class="text-muted-foreground block"
                                        >Vencimento</span
                                    >
                                    <span class="text-white"
                                        >{formatDate(item.due_date)}</span
                                    >
                                </div>
                                <div>
                                    <span class="text-muted-foreground block"
                                        >Principal</span
                                    >
                                    <span class="text-white"
                                        >{formatCurrency(
                                            item.principal_value,
                                        )}</span
                                    >
                                </div>
                            </div>
                        </Card.Content>

                        <Card.Footer
                            class="bg-black/20 border-t border-white/5 p-4 flex justify-end gap-2"
                        >
                            <Button
                                variant="outline"
                                size="sm"
                                onclick={() => openViewModal(item)}
                            >
                                <Eye class="w-4 h-4 mr-2" />
                            </Button>
                            <Button
                                variant="default"
                                size="sm"
                                class="bg-green-600 hover:bg-green-700 text-white flex-1"
                                onclick={() => openPayModal(item)}
                            >
                                <DollarSign class="w-4 h-4 mr-2" /> Pagar
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon"
                                onclick={() => deleteDarf(item)}
                            >
                                <Trash2 class="w-4 h-4 text-red-400" />
                            </Button>
                        </Card.Footer>
                    </Card.Root>
                {/each}
            </div>
        </div>
    {/if}

    <!-- History/Paid DARFs (List) -->
    <div class="mt-8">
        <h3
            class="text-lg font-semibold text-white mb-4 flex items-center gap-2"
        >
            <CheckCircle class="w-5 h-5 text-green-500" />
            Histórico de Pagamentos ({year})
        </h3>

        {#if irpfStore.loading}
            <div class="p-8 text-center text-muted-foreground">
                Carregando...
            </div>
        {:else if historyDarfs.length === 0}
            <div
                class="p-8 text-center border border-dashed border-white/10 rounded-lg text-muted-foreground"
            >
                Nenhum histórico encontrado para {year}.
            </div>
        {:else}
            <div
                class="bg-black/40 border border-white/10 rounded-lg overflow-hidden glass"
            >
                <table class="w-full text-sm text-left">
                    <thead
                        class="bg-black/20 text-xs uppercase text-muted-foreground border-b border-white/5"
                    >
                        <tr>
                            <th class="px-6 py-3">Período</th>
                            <th class="px-6 py-3">Cód. Receita</th>
                            <th class="px-6 py-3 text-right">Valor Principal</th
                            >
                            <th class="px-6 py-3 text-right">Multa/Juros</th>
                            <th class="px-6 py-3 text-right">Total Pago</th>
                            <th class="px-6 py-3">Status</th>
                            <th class="px-6 py-3 text-right">Ações</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-white/5">
                        {#each historyDarfs as item}
                            <tr
                                class="hover:bg-white/5 transition-colors group"
                            >
                                <td class="px-6 py-4 font-medium text-white"
                                    >{item.period}</td
                                >
                                <td class="px-6 py-4">{item.revenue_code}</td>
                                <td
                                    class="px-6 py-4 text-right font-mono text-muted-foreground"
                                    >{formatCurrency(item.principal_value)}</td
                                >
                                <td
                                    class="px-6 py-4 text-right font-mono text-red-300"
                                >
                                    {#if item.fine + item.interest > 0}
                                        {formatCurrency(
                                            item.fine + item.interest,
                                        )}
                                    {:else}
                                        -
                                    {/if}
                                </td>
                                <td
                                    class="px-6 py-4 text-right font-mono font-bold text-green-400"
                                    >{formatCurrency(item.total_value)}</td
                                >
                                <td class="px-6 py-4">
                                    <span
                                        class="px-2 py-1 rounded-full text-xs font-bold bg-green-500/10 text-green-500 border border-green-500/20 flex w-fit items-center gap-1"
                                    >
                                        <CheckCircle class="w-3 h-3" /> Pago
                                    </span>
                                </td>
                                <td class="px-6 py-4 text-right">
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        class="h-8 w-8 p-0"
                                        onclick={() => openViewModal(item)}
                                    >
                                        <Eye
                                            class="w-4 h-4 text-slate-400 group-hover:text-white transition-colors"
                                        />
                                    </Button>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        class="h-8 w-8 p-0"
                                        onclick={() => deleteDarf(item)}
                                    >
                                        <Trash2
                                            class="w-4 h-4 text-slate-400 group-hover:text-red-400 transition-colors"
                                        />
                                    </Button>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}

        <!-- Payment Modal -->
        <Dialog.Root bind:open={isPayModalOpen}>
            <Dialog.Content class="sm:max-w-[425px]">
                <Dialog.Header>
                    <Dialog.Title>Registrar Pagamento</Dialog.Title>
                    <Dialog.Description
                        >Informe os dados do pagamento e a conta de origem.</Dialog.Description
                    >
                </Dialog.Header>
                <div class="grid gap-4 py-4">
                    <div class="space-y-1">
                        <Label>Valor Principal</Label>
                        <div class="font-mono text-xl text-white">
                            {formatCurrency(paymentData.principal)}
                        </div>
                    </div>

                    <div class="space-y-2">
                        <Label>Conta de Origem</Label>
                        <div class="flex flex-col gap-2">
                            <select
                                bind:value={paymentData.accountId}
                                class="flex h-10 w-full rounded-md border border-white/10 bg-black/20 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-black"
                            >
                                <option value="">Selecione uma conta...</option>
                                {#each settingsStore.accounts as acc}
                                    <option value={acc.id}
                                        >{acc.nickname} ({acc.currency})</option
                                    >
                                {/each}
                            </select>
                            {#if !paymentData.accountId}
                                <span class="text-xs text-red-400"
                                    >Selecione uma conta para debitar.</span
                                >
                            {/if}
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2">
                            <Label>Multa (R$)</Label>
                            <Input
                                type="number"
                                step="0.01"
                                bind:value={paymentData.fine}
                                oninput={updatePaymentTotal}
                                class="bg-black/20 border-white/10 text-white"
                            />
                        </div>
                        <div class="space-y-2">
                            <Label>Juros (R$)</Label>
                            <Input
                                type="number"
                                step="0.01"
                                bind:value={paymentData.interest}
                                oninput={updatePaymentTotal}
                                class="bg-black/20 border-white/10 text-white"
                            />
                        </div>
                    </div>
                    <div class="space-y-2">
                        <Label>Data do Pagamento</Label>
                        <Input
                            type="date"
                            bind:value={paymentData.date}
                            class="bg-black/20 border-white/10 text-white"
                        />
                    </div>
                    <div
                        class="space-y-1 pt-4 border-t border-white/10 flex justify-between items-end"
                    >
                        <Label class="text-base">Valor Total Pago</Label>
                        <div class="font-mono text-2xl font-bold text-primary">
                            {formatCurrency(paymentData.total)}
                        </div>
                    </div>
                </div>
                <Dialog.Footer>
                    <Button
                        variant="outline"
                        onclick={() => (isPayModalOpen = false)}
                        >Cancelar</Button
                    >
                    <Button
                        class="neon-glow bg-primary text-black font-bold"
                        disabled={!paymentData.accountId}
                        onclick={confirmPayment}
                    >
                        <CheckCircle class="w-4 h-4 mr-2" /> Confirmar Pagamento
                    </Button>
                </Dialog.Footer>
            </Dialog.Content>
        </Dialog.Root>

        <!-- View Modal -->
        <Dialog.Root bind:open={isViewModalOpen}>
            <Dialog.Content class="sm:max-w-[425px]">
                <Dialog.Header>
                    <Dialog.Title>Detalhes da DARF</Dialog.Title>
                </Dialog.Header>
                {#if selectedDarf}
                    <div class="grid gap-4 py-4">
                        <div class="space-y-1">
                            <Label>Código Receita</Label>
                            <div class="font-mono text-white">
                                {selectedDarf.revenue_code}
                            </div>
                        </div>
                        <div class="space-y-1">
                            <Label>Valor Principal</Label>
                            <div class="font-mono text-xl text-primary">
                                {formatCurrency(selectedDarf.principal_value)}
                            </div>
                        </div>

                        <!-- Appraisal Snapshot (Simulation/Link) -->
                        <div
                            class="p-3 bg-white/5 rounded border border-white/10 space-y-2"
                        >
                            <div class="flex items-center justify-between">
                                <span class="text-xs text-muted-foreground"
                                    >Período de Apuração</span
                                >
                                <span class="text-sm font-bold text-white"
                                    >{selectedDarf.period}</span
                                >
                            </div>
                            <Button
                                variant="secondary"
                                size="sm"
                                class="w-full text-xs"
                                href="/fiscal/irpf"
                            >
                                <Eye class="w-3 h-3 mr-2" />
                                Ver Apuração Detalhada (Dashboard)
                            </Button>
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div class="space-y-1">
                                <Label>Multa</Label>
                                <div class="font-mono text-white">
                                    {formatCurrency(selectedDarf.fine)}
                                </div>
                            </div>
                            <div class="space-y-1">
                                <Label>Juros</Label>
                                <div class="font-mono text-white">
                                    {formatCurrency(selectedDarf.interest)}
                                </div>
                            </div>
                        </div>
                        <div class="space-y-1 pt-2 border-t border-white/10">
                            <Label>Valor Total</Label>
                            <div class="font-mono text-xl font-bold text-white">
                                {formatCurrency(selectedDarf.total_value)}
                            </div>
                        </div>
                        {#if selectedDarf.status === "Paid"}
                            <div
                                class="p-3 bg-green-500/10 border border-green-500/20 rounded text-green-400 text-sm flex flex-col gap-1"
                            >
                                <div class="flex items-center gap-2 font-bold">
                                    <CheckCircle class="w-4 h-4" />
                                    Pago em {formatDate(
                                        selectedDarf.payment_date,
                                    )}
                                </div>
                                {#if selectedDarf.account_id}
                                    <div class="text-xs text-green-400/70 pl-6">
                                        Via: {getAccountName(
                                            selectedDarf.account_id,
                                        )}
                                    </div>
                                {/if}

                                <Button
                                    variant="secondary"
                                    size="sm"
                                    class="w-full mt-2"
                                    onclick={() => openUnpayModal(selectedDarf)}
                                >
                                    <Undo2 class="w-3 h-3 mr-2" /> Desfazer Pagamento
                                </Button>
                            </div>
                        {/if}
                    </div>
                {/if}
            </Dialog.Content>
        </Dialog.Root>

        <!-- Delete Modal -->
        <Dialog.Root bind:open={isDeleteModalOpen}>
            <Dialog.Content class="sm:max-w-[400px]">
                <Dialog.Header>
                    <Dialog.Title>Confirmar Exclusão</Dialog.Title>
                    <Dialog.Description
                        >Tem certeza que deseja excluir esta DARF?</Dialog.Description
                    >
                </Dialog.Header>
                <Dialog.Footer>
                    <Button
                        variant="outline"
                        onclick={() => (isDeleteModalOpen = false)}
                        >Cancelar</Button
                    >
                    <Button variant="destructive" onclick={confirmDelete}
                        >Excluir</Button
                    >
                </Dialog.Footer>
            </Dialog.Content>
        </Dialog.Root>

        <!-- Unpay Modal -->
        <Dialog.Root bind:open={isUnpayModalOpen}>
            <Dialog.Content class="sm:max-w-[400px]">
                <Dialog.Header>
                    <Dialog.Title>Desfazer Pagamento</Dialog.Title>
                    <Dialog.Description>
                        Isso irá reverter o status da DARF para <b>Pendente</b>
                        e estornar o valor de
                        <b
                            >{darfToUnpay
                                ? formatCurrency(darfToUnpay.total_value)
                                : ""}</b
                        >
                        para a conta de origem.
                        <br /><br />
                        A transação de saída será removida do Hub Financeiro.
                    </Dialog.Description>
                </Dialog.Header>
                <Dialog.Footer>
                    <Button
                        variant="outline"
                        onclick={() => (isUnpayModalOpen = false)}
                        >Cancelar</Button
                    >
                    <Button variant="default" onclick={confirmUnpay}
                        >Confirmar Estorno</Button
                    >
                </Dialog.Footer>
            </Dialog.Content>
        </Dialog.Root>
    </div>
</div>
