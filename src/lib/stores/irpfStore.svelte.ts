import { invoke } from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";
import { settingsStore } from "./settings.svelte";

export interface TaxAppraisal {
    id?: string;
    period_month: number;
    period_year: number;
    trade_type: "DayTrade" | "SwingTrade" | string;
    tax_rule_id: string; // Link specific rule
    revenue_code: string; // "6015" or "3317"
    gross_profit: number;
    loss: number;
    net_profit: number;
    compensated_loss: number;
    calculation_basis: number;
    tax_rate: number;
    tax_due: number;
    withheld_tax: number;
    tax_payable: number;
    tax_accumulated: number;
    total_payable: number;
    withholding_credit_used: number;
    withholding_credit_remaining: number;
    is_exempt: boolean;
    calculation_date: string;
    status: "Pending" | "Paid" | "Ok";
    trade_ids: string[];
}

export interface TaxLoss {
    id?: string;
    trade_type: "DayTrade" | "SwingTrade";
    amount: number;
    origin_date: string;
    balance: number;
}

export interface TaxDarf {
    id?: string;
    appraisal_id: string;
    revenue_code: string;
    period: string; // MM/YYYY
    principal_value: number;
    fine: number;
    interest: number;
    total_value: number;
    due_date: string;
    payment_date?: string;
    status: "Pending" | "Paid";
    darf_number?: string;
    account_id?: string;
    transaction_id?: string;
}

class IrpfStore {
    appraisals = $state<TaxAppraisal[]>([]);
    losses = $state<TaxLoss[]>([]);
    darfs = $state<TaxDarf[]>([]);
    loading = $state(false);
    selectedYear = $state(new Date().getFullYear());

    // Computed KPIs for the selected year
    totalDue = $derived(
        this.appraisals
            .filter((a) => a.period_year === this.selectedYear)
            .reduce((acc, a) => acc + a.total_payable, 0),
    );

    totalPaid = $derived(
        this.darfs
            .filter((d) => {
                const parts = d.period.split("/");
                const y = parts.length > 1 ? parseInt(parts[1]) : 0;
                return y === this.selectedYear && d.status === "Paid";
            })
            .reduce((acc, d) => acc + d.total_value, 0),
    );

    pendingGuiasCount = $derived(
        this.darfs.filter((d) => {
            const parts = d.period.split("/");
            const y = parts.length > 1 ? parseInt(parts[1]) : 0;
            return y === this.selectedYear && d.status === "Pending";
        }).length,
    );

    pendingAmount = $derived(
        this.darfs
            .filter((d) => {
                const parts = d.period.split("/");
                const y = parts.length > 1 ? parseInt(parts[1]) : 0;
                return y === this.selectedYear && d.status === "Pending";
            })
            .reduce((acc, d) => acc + d.total_value, 0) +
        this.appraisals
            .filter(
                (a) =>
                    a.status === "Pending" &&
                    a.period_year === this.selectedYear &&
                    !this.darfs.some(
                        (d) => this.getId(d.appraisal_id) === this.getId(a.id),
                    ),
            )
            .reduce((acc, a) => acc + a.total_payable, 0),
    );

    async loadAllData(year?: number) {
        const y = year || this.selectedYear;
        if (year) this.selectedYear = year;

        await Promise.all([
            this.loadAppraisals(y),
            this.loadDarfs(y),
            this.loadAccumulatedLosses()
        ]);
    }

    async loadAppraisals(year: number) {
        this.loading = true;
        try {
            this.appraisals = await invoke("get_appraisals", { year });
        } catch (error) {
            console.error("Failed to load appraisals:", error);
            toast.error("Erro ao carregar apurações");
        } finally {
            this.loading = false;
        }
    }

    async calculateMonthlyTax(month: number, year: number) {
        console.log(`[IRPF_STORE] calculateMonthlyTax called for month=${month}, year=${year}`);
        this.loading = true;
        try {
            console.log(`[IRPF_STORE] Invoking calculate_monthly_tax...`);
            const results = await invoke<TaxAppraisal[]>("calculate_monthly_tax", { month, year });
            console.log(`[IRPF_STORE] Received results:`, results);
            if (results.length === 0) {
                toast.info("Nenhuma operação encontrada para este período.");
            } else {
                this.appraisals = [...this.appraisals.filter(a => !(a.period_month === month && a.period_year === year)), ...results];
                // Sort by month desc
                this.appraisals.sort((a, b) => {
                    if (a.period_year !== b.period_year) return b.period_year - a.period_year;
                    return b.period_month - a.period_month;
                });
                toast.success("Apuração realizada com sucesso!");
            }
            return results;
        } catch (error) {
            console.error("Failed to calculate tax:", error);
            toast.error("Erro ao calcular imposto: " + String(error));
            throw error;
        } finally {
            this.loading = false;
        }
    }

    async saveAppraisal(data: TaxAppraisal) {
        try {
            const saved = await invoke<TaxAppraisal>("save_appraisal", { data });
            const index = this.appraisals.findIndex(
                (a) => a.period_month === saved.period_month && a.period_year === saved.period_year && a.trade_type === saved.trade_type
            );
            if (index !== -1) {
                this.appraisals[index] = saved;
            } else {
                this.appraisals.push(saved);
            }
            // Sort after save to maintain order
            this.appraisals.sort((a, b) => {
                if (a.period_year !== b.period_year) return b.period_year - a.period_year;
                return b.period_month - a.period_month;
            }); toast.success(`Apuração de ${data.trade_type} salva com sucesso!`);
            this.loadAccumulatedLosses(); // Refresh losses as they might have changed
            return saved;
        } catch (error) {
            console.error("Failed to save appraisal:", error);
            toast.error("Erro ao salvar apuração: " + String(error));
            throw error;
        }
    }

    async deleteAppraisal(id: string) {
        try {
            await invoke("delete_appraisal", { id });
            this.appraisals = this.appraisals.filter(a => this.getId(a.id) !== id);
            toast.success("Apuração excluída com sucesso!");
            this.loadAccumulatedLosses();
        } catch (error) {
            console.error("Failed to delete appraisal:", error);
            toast.error("Erro ao excluir: " + String(error));
        }
    }

    async loadAccumulatedLosses() {
        try {
            this.losses = await invoke("get_accumulated_losses");
        } catch (error) {
            console.error("Failed to load losses:", error);
        }
    }

    async deleteLoss(id: string) {
        try {
            await invoke("delete_tax_loss", { id });
            this.losses = this.losses.filter(l => this.getId(l.id) !== id);
            toast.success("Prejuízo removido!");
        } catch (error) {
            console.error("Failed to delete loss:", error);
            toast.error("Erro ao remover prejuízo: " + String(error));
        }
    }

    async loadDarfs(year: number) {
        this.loading = true;
        try {
            this.darfs = await invoke("get_darfs", { year });
        } catch (error) {
            console.error("Failed to load DARFs:", error);
            toast.error("Erro ao carregar DARFs");
        } finally {
            this.loading = false;
        }
    }

    async generateDarf(appraisalId: string) {
        try {
            const darf = await invoke<TaxDarf>("generate_darf", { appraisalId });
            this.darfs = [darf, ...this.darfs];
            toast.success("DARF gerada com sucesso!");
            // Refresh appraisals to update status if needed
            const year = parseInt(darf.period.split('/')[1]);
            this.loadAppraisals(year);
            return darf;
        } catch (error) {
            console.error("Failed to generate DARF:", error);
            toast.error("Erro ao gerar DARF: " + String(error));
            throw error;
        }
    }

    async markDarfPaid(id: string, paymentDate: string, paidValue: number, accountId: string, fine?: number, interest?: number) {
        try {
            const transactionId = crypto.randomUUID();
            const updated = await invoke<TaxDarf>("mark_darf_paid", {
                id,
                paymentDate,
                paidValue,
                fine,
                interest,
                accountId,
                transactionId
            });
            const index = this.darfs.findIndex(d => this.getId(d.id) === this.getId(updated.id));
            if (index !== -1) this.darfs[index] = updated;

            toast.success("DARF marcada como paga e registrada no Financeiro!");

            // Refresh appraisals to update status
            const year = parseInt(updated.period.split('/')[1]);
            this.loadAppraisals(year);
            this.loadDarfs(year);

            // REFRESH FINANCE HUB
            settingsStore.loadData();

            return updated;
        } catch (error) {
            console.error("Failed to mark DARF paid:", error);
            toast.error("Erro ao atualizar DARF: " + String(error));
            throw error;
        }
    }

    async getDarfByTransaction(transactionId: string): Promise<TaxDarf | null> {
        try {
            return await invoke<TaxDarf | null>("get_darf_by_transaction", { transactionId });
        } catch (error) {
            console.error("Failed to fetch DARF by transaction:", error);
            return null;
        }
    }

    async getDarfById(id: string): Promise<TaxDarf | null> {
        try {
            return await invoke<TaxDarf | null>("get_darf_by_id", { id });
        } catch (error) {
            console.error("Failed to fetch DARF by ID:", error);
            return null;
        }
    }

    async getAppraisalById(id: string): Promise<TaxAppraisal | null> {
        try {
            return await invoke<TaxAppraisal | null>("get_appraisal_by_id", { id });
        } catch (error) {
            console.error("Failed to fetch appraisal by ID:", error);
            return null;
        }
    }

    async diagnosticDumpDarfs() {
        try {
            await invoke("diagnostic_dump_darfs");
            console.log("[DIAGNOSTIC] Command sent. Check backend terminal for output.");
        } catch (error) {
            console.error("Diagnostic command failed:", error);
        }
    }

    async deleteDarf(id: string) {
        try {
            await invoke("delete_darf", { id });
            this.darfs = this.darfs.filter(d => this.getId(d.id) !== id);
            toast.success("DARF excluída!");
            // Refresh appraisals to update status (might revert to pending? backend doesn't seem to revert status automatically in delete_darf, but we can re-fetch)
            // Ideally backend should revert appraisal status.
        } catch (error) {
            console.error("Failed to delete DARF:", error);
            toast.error("Erro ao excluir DARF: " + String(error));
        }
    }

    async unpayDarf(id: string) {
        try {
            const updated = await invoke<TaxDarf>("unpay_darf", { id });

            const index = this.darfs.findIndex(d => this.getId(d.id) === this.getId(updated.id));
            if (index !== -1) {
                this.darfs[index] = updated;
            }

            toast.success("Pagamento desfeito e valor estornado!");

            // Refresh appraisals
            const year = parseInt(updated.period.split('/')[1]);
            this.loadAppraisals(year);
            this.loadDarfs(year);

            // REFRESH FINANCE HUB
            settingsStore.loadData();

            return updated;
        } catch (error) {
            console.error("Failed to unpay DARF:", error);
            toast.error("Erro ao desfazer pagamento: " + String(error));
            throw error;
        }
    }

    async resetData() {
        try {
            await invoke("reset_irpf_data");
            this.appraisals = [];
            this.losses = [];
            this.darfs = [];
            toast.success("Todos os dados fiscais foram resetados.");
        } catch (error) {
            console.error("Failed to reset data:", error);
            toast.error("Erro ao resetar dados: " + String(error));
        }
    }

    // Helper to extract ID string from SurrealDB Thing or Object
    getId(id: any): string {
        if (!id) return "";
        let idStr = "";

        if (typeof id === "string") {
            idStr = id;
        } else if (typeof id === "object") {
            if (id.String) idStr = id.String;
            else if (id.id) {
                if (typeof id.id === "object" && id.id.String) idStr = id.id.String;
                else if (typeof id.id === "string") idStr = id.id;
                else idStr = String(id.id);
            } else if (id.tb && id.id) {
                if (typeof id.id === "object" && id.id.String) idStr = id.id.String;
                else idStr = String(id.id);
            } else {
                idStr = String(id);
            }
        } else {
            idStr = String(id);
        }

        if (idStr.includes(":")) {
            return idStr.split(":").pop() || idStr;
        }
        return idStr;
    }
}

export const irpfStore = new IrpfStore();
