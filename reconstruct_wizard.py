
import os

file_path = r'c:\PROJETOS\TraderLogPro\src\lib\components\trades\NewTradeWizard.svelte'

# Corrected Middle Content (Raw string to avoid any interpretation)
functions = r"""
    function handleNext() {
        if (currentStep === 1) {
            if (
                !formData.account_id ||
                !formData.asset ||
                !formData.strategy_id ||
                !formData.modality_id
            ) {
                toast.error($t("trades.wizard.errors.required_fields"));
                return;
            }
            if (formData.entry_price <= 0) {
                toast.error($t("trades.wizard.messages.entry_price_required"));
                return;
            }
        }
        if (currentStep < steps.length) currentStep++;
    }

    function handlePrev() {
        if (currentStep > 1) currentStep--;
    }

    async function handleSubmit() {
        console.log(
            "[NewTradeWizard] Submitting form. Mode:",
            trade?.id ? "Edit" : "New",
        );
        console.log("[NewTradeWizard] Form Data:", $state.snapshot(formData));

        // CRITICAL: Ensure asset_type_id is set before saving
        if (!selectedAssetTypeId && formData.asset) {
            const asset = assetsList.find(
                (a) => a.symbol.toUpperCase() === formData.asset.toUpperCase(),
            );
            if (asset) {
                const type = assetTypesList.find(
                    (t) =>
                        t.id === asset.asset_type_id ||
                        t.id.replace(/^asset_type:/, "") ===
                            asset.asset_type_id.replace(/^asset_type:/, ""),
                );
                selectedAssetTypeId = type ? type.id : asset.asset_type_id;
                console.log(
                    "[NewTradeWizard] Auto-filled asset_type_id from asset:",
                    selectedAssetTypeId,
                );
            } else {
                toast.error("Por favor, selecione um Tipo de Ativo válido.");
                return;
            }
        }

        if (!selectedAssetTypeId) {
            toast.error(
                "Tipo de Ativo é obrigatório. Por favor, selecione um.",
            );
            return;
        }

        isSubmitting = true;
        try {
            const tradeData: any = {
                // Save as ISO string to preserve time for editing later
                date: (formData.entry_date as string)?.includes("T")
                    ? formData.entry_date + ":00Z"
                    : formData.entry_date + "T00:00:00Z",
                asset_symbol: formData.asset,
                asset_type_id: selectedAssetTypeId,
                strategy_id: formData.strategy_id,
                account_id: formData.account_id,
                result: calculationResult.net,
                quantity: formData.quantity,
                direction: formData.direction === "buy" ? "Buy" : "Sell",
                entry_price: formData.entry_price,
                exit_price: formData.exit_price,
                exit_date: formData.exit_date
                    ? (formData.exit_date as string).includes("T")
                        ? formData.exit_date + ":00Z"
                        : formData.exit_date + "T00:00:00Z"
                    : formData.exit_price !== null
                      ? new Date().toISOString() // Fallback to now if closed but no date
                      : null,
                fee_total: formData.fees,
                notes: formData.entry_rationale,

                timeframe: formData.timeframe,
                volatility: formData.volatility,
                modality_id: formData.modality_id,
                stop_loss: formData.stop_loss,
                take_profit: formData.take_profit,
                intensity: formData.intensity,

                entry_emotional_state_id: formData.entry_emotional_state_id,

                exit_reason: formData.exit_reason,
                exit_emotional_state_id: formData.exit_emotional_state_id,

                entry_rationale: formData.entry_rationale,
                confirmation_signals: formData.confirmation_signals,
                market_context: formData.market_context,
                relevant_news: formData.relevant_news,

                followed_plan: !!formData.followed_plan,
                what_worked: formData.what_worked,
                mistakes_improvements: formData.mistakes_improvements,
                lessons_learned: formData.lessons_learned,

                images: formData.images,
                partial_exits: formData.partial_exits,
            };

            console.log(
                "[NewTradeWizard] Submission tradeData:",
                JSON.stringify(tradeData, null, 2),
            );

            if (trade?.id) {
                console.log(
                    "[NewTradeWizard] Calling updateTrade for ID:",
                    trade.id,
                );
                const result = await tradesStore.updateTrade(
                    trade.id,
                    tradeData,
                );
                if (result.success) {
                    toast.success(
                        $t("trades.wizard.messages.update_success") ||
                            "Operação atualizada!",
                    );
                    onsave();
                    close();
                } else {
                    console.error(
                        "[NewTradeWizard] Backend Update Error:",
                        result.error,
                    );
                    toast.error(result.error || "Erro ao atualizar operação.");
                }
            } else {
                console.log("[NewTradeWizard] Calling addTrade");
                const result = await tradesStore.addTrade(tradeData);
                if (result.success) {
                    toast.success($t("trades.wizard.messages.save_success"));
                    onsave();
                    close();
                } else {
                    console.error(
                        "[NewTradeWizard] Backend Save Error:",
                        result.error,
                    );
                    toast.error(result.error || "Erro ao salvar operação.");
                }
            }
        } catch (e) {
            console.error("[NewTradeWizard] CRITICAL CLIENT CRASH:", e);
            toast.error($t("trades.wizard.messages.save_error"));
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="flex flex-col h-full bg-background overflow-hidden">
"""

with open(file_path, 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Search for the start of the broken section (it was lines 580+)
# Let's find "const calculationResult = $derived.by" which should be before.
head_limit = -1
for i, line in enumerate(lines):
    if 'const calculationResult = $derived.by' in line:
        # Keep until the end of that block
        continue
    if 'function handleNext()' in line:
        head_limit = i
        break

if head_limit == -1:
    # Fallback to the known line count if exact string match fails
    head_limit = 581 

head = lines[:head_limit]

# Search for the start of the template tail in the current corrupted file
tail_start = -1
for i, line in enumerate(lines):
    if '<!-- Header with Modern Stepper -->' in line:
        tail_start = i
        break

if tail_start != -1:
    tail = lines[tail_start:]
    final_content = "".join(head) + functions + "\n" + "".join(tail)
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(final_content)
    print("Success")
else:
    print("Failed to find tail marker")
