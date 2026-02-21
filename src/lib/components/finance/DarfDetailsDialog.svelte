<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { Separator } from "$lib/components/ui/separator";
    import { irpfStore, type TaxDarf } from "$lib/stores/irpfStore.svelte";
    import {
        FileText,
        Calendar,
        Wallet,
        Percent,
        Info,
        CheckCircle2,
    } from "lucide-svelte";
    import { format } from "date-fns";
    import { ptBR } from "date-fns/locale";

    let { transactionId = "", open = $bindable(false) } = $props();

    let darf = $state<TaxDarf | null>(null);
    let loading = $state(false);

    $effect(() => {
        if (open && transactionId) {
            loadDarf();
        }
    });

    async function loadDarf() {
        loading = true;
        darf = await irpfStore.getDarfByTransaction(transactionId);
        loading = false;
    }

    function formatCurrency(value: number) {
        return new Intl.NumberFormat("pt-BR", {
            style: "currency",
            currency: "BRL",
        }).format(value);
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content
        class="sm:max-w-[500px] bg-zinc-950 border-zinc-800 text-zinc-100"
    >
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2 text-xl">
                <FileText class="w-5 h-5 text-amber-500" />
                Detalhes do Pagamento DARF
            </Dialog.Title>
            <Dialog.Description class="text-zinc-400">
                Informações detalhadas sobre o imposto recolhido.
            </Dialog.Description>
        </Dialog.Header>

        {#if loading}
            <div class="py-12 flex flex-col items-center justify-center gap-4">
                <div
                    class="w-8 h-8 border-4 border-amber-500/20 border-t-amber-500 rounded-full animate-spin"
                ></div>
                <p class="text-sm text-zinc-500">Buscando dados do DARF...</p>
            </div>
        {:else if darf}
            <div class="space-y-6 pt-4">
                <!-- Status & Period -->
                <div class="flex items-center justify-between">
                    <div class="space-y-1">
                        <span
                            class="text-xs text-zinc-500 uppercase font-bold tracking-wider"
                            >Período de Apuração</span
                        >
                        <div class="flex items-center gap-2">
                            <Calendar class="w-4 h-4 text-zinc-400" />
                            <span class="text-lg font-medium"
                                >{darf.period}</span
                            >
                        </div>
                    </div>
                    <Badge
                        variant="outline"
                        class="bg-green-500/10 text-green-500 border-green-500/20 px-3 py-1"
                    >
                        <CheckCircle2 class="w-3.5 h-3.5 mr-1.5" />
                        Pago
                    </Badge>
                </div>

                <Separator class="bg-zinc-800" />

                <!-- Codes Section -->
                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1">
                        <span
                            class="text-xs text-zinc-500 uppercase font-bold tracking-wider"
                            >Código da Receita</span
                        >
                        <div class="text-base font-semibold text-zinc-200">
                            {darf.revenue_code}
                        </div>
                    </div>
                    <div class="space-y-1">
                        <span
                            class="text-xs text-zinc-500 uppercase font-bold tracking-wider"
                            >Vencimento</span
                        >
                        <div class="text-base font-semibold text-zinc-200">
                            {darf.due_date
                                ? format(new Date(darf.due_date), "dd/MM/yyyy")
                                : "-"}
                        </div>
                    </div>
                </div>

                <!-- Values Breakdown -->
                <div
                    class="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800/50 space-y-3"
                >
                    <div class="flex justify-between items-center text-sm">
                        <span class="text-zinc-400">Valor do Principal</span>
                        <span class="font-mono text-zinc-200"
                            >{formatCurrency(darf.principal_value)}</span
                        >
                    </div>

                    {#if darf.fine > 0}
                        <div class="flex justify-between items-center text-sm">
                            <span
                                class="text-zinc-400 flex items-center gap-1.5"
                            >
                                Multa <Percent class="w-3 h-3" />
                            </span>
                            <span class="font-mono text-amber-500"
                                >+{formatCurrency(darf.fine)}</span
                            >
                        </div>
                    {/if}

                    {#if darf.interest > 0}
                        <div class="flex justify-between items-center text-sm">
                            <span
                                class="text-zinc-400 flex items-center gap-1.5"
                            >
                                Juros <Info class="w-3 h-3" />
                            </span>
                            <span class="font-mono text-amber-500"
                                >+{formatCurrency(darf.interest)}</span
                            >
                        </div>
                    {/if}

                    <div
                        class="pt-2 border-t border-zinc-800 flex justify-between items-center"
                    >
                        <span class="font-bold text-zinc-100">Total Pago</span>
                        <span
                            class="text-lg font-bold text-green-400 font-mono"
                        >
                            {formatCurrency(darf.total_value)}
                        </span>
                    </div>
                </div>

                <!-- Payment Info -->
                <div
                    class="flex items-center gap-3 p-3 bg-zinc-900/30 rounded-lg border border-zinc-800/30"
                >
                    <Wallet class="w-5 h-5 text-zinc-500" />
                    <div class="space-y-0.5">
                        <p
                            class="text-[10px] text-zinc-500 uppercase font-bold"
                        >
                            Pago em
                        </p>
                        <p class="text-xs text-zinc-300">
                            {darf.payment_date
                                ? format(
                                      new Date(darf.payment_date),
                                      "dd 'de' MMMM 'de' yyyy",
                                      { locale: ptBR },
                                  )
                                : "Data não disponível"}
                        </p>
                    </div>
                </div>
            </div>
        {:else}
            <div class="py-12 text-center space-y-2">
                <Info class="w-8 h-8 text-zinc-600 mx-auto" />
                <p class="text-zinc-500">
                    Não foi possível carregar os detalhes deste DARF.
                </p>
            </div>
        {/if}

        <Dialog.Footer class="mt-4">
            <Button
                variant="secondary"
                onclick={() => (open = false)}
                class="bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border-zinc-700"
            >
                Fechar
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
