<script lang="ts">
    import * as Table from "$lib/components/ui/table";
    import { Badge } from "$lib/components/ui/badge";
    import { cn } from "$lib/utils";
    import { t, locale } from "svelte-i18n";

    let { trades, currency } = $props<{
        trades: any[];
        currency: string;
    }>();

    function formatCurrency(val: number, curr: string) {
        try {
            return new Intl.NumberFormat($locale || "pt-BR", {
                style: "currency",
                currency: curr,
            }).format(val);
        } catch (e) {
            return `${curr} ${val.toLocaleString($locale || "pt-BR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
        }
    }
</script>

<div class="border border-border/40 rounded-xl overflow-hidden bg-muted/20">
    <Table.Root>
        <Table.Header class="bg-muted/40">
            <Table.Row class="hover:bg-transparent border-border/50">
                <Table.Head class="text-[10px] font-black text-muted-foreground uppercase tracking-widest px-4 h-10">
                    {$t("finance.details.columns.asset")}
                </Table.Head>
                <Table.Head class="text-[10px] font-black text-muted-foreground uppercase tracking-widest h-10">
                    {$t("finance.details.columns.side")}
                </Table.Head>
                <Table.Head class="text-right text-[10px] font-black text-muted-foreground uppercase tracking-widest px-4 h-10">
                    {$t("finance.details.columns.result")}
                </Table.Head>
            </Table.Row>
        </Table.Header>
        <Table.Body>
            {#if trades && trades.length > 0}
                {#each trades as trade}
                    <Table.Row class="border-border/40 hover:bg-muted/30 transition-colors">
                        <Table.Cell class="font-black text-foreground/90 font-mono tracking-tighter px-4 py-3">
                            {trade.asset_symbol}
                        </Table.Cell>
                        <Table.Cell class="py-3">
                            <Badge
                                variant="outline"
                                class={cn(
                                    "text-[9px] font-black uppercase tracking-widest px-2 py-0.5",
                                    trade.direction === "Buy"
                                        ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20"
                                        : "bg-rose-500/10 text-rose-500 border-rose-500/20",
                                )}
                            >
                                {trade.direction.toUpperCase()}
                            </Badge>
                        </Table.Cell>
                        <Table.Cell class="text-right font-mono font-black py-3 px-4 {trade.result >= 0 ? 'text-emerald-500' : 'text-rose-500'}">
                            {formatCurrency(trade.result, currency)}
                        </Table.Cell>
                    </Table.Row>
                {/each}
            {:else}
                <Table.Row>
                    <Table.Cell colspan={3} class="text-center h-24 text-muted-foreground">
                        <div class="flex flex-col items-center gap-2 opacity-50">
                            <span class="text-[10px] font-black uppercase tracking-widest">
                                {$t("finance.details.noTrades")}
                            </span>
                        </div>
                    </Table.Cell>
                </Table.Row>
            {/if}
        </Table.Body>
    </Table.Root>
</div>
