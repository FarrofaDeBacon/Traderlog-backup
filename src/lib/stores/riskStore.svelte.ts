// --- RISK STORE (Position Sizing Enabled) ---
// This file serves as the reactive bindings for the pure risk domains.

import { settingsStore } from './settings.svelte';
import { tradesStore } from './trades.svelte';
import { 
    buildRiskCockpitState, 
    type RiskCockpitState,
    adaptPositionSizingInput,
    calculatePositionSizing,
    type PositionSizingInput,
    type PositionSizingResult
} from '$lib/domain/risk';
import { 
    adaptSettingsProfileToDomain, 
    adaptTradeToDomain, 
    adaptGrowthPhaseToDomain 
} from '$lib/domain/risk/risk-adapters';
import type { ResolvedGrowthContext } from '../types';

/**
 * Store reativo estritamente focado no Cockpit de Risco.
 * Consome os stores de `settings` e `trades`, converte usando `risk-adapters`
 * e expõe o resultado do domínio em tempo real via getter derivado.
 */
class RiskStore {
    /**
     * Ativo atualmente selecionado pelo usuário na UI (ex: na boleta)
     * Necessário para alimentar o contexto do Position Sizing Engine.
     */
    activeAssetId = $state<string | null>(null);

    /**
     * Estado agregado final do Cockpit de Risco calculado pelo Domínio Puro.
     * Nenhuma regra de negócio é tomada AQUI, apenas injeção limpa de dependências.
     */
    get riskCockpitState(): RiskCockpitState | null {
        // 1. Obter Perfil Ativo
        const activeProfile = settingsStore.activeProfile;
        if (!activeProfile || !activeProfile.active) return null;

        // 2. Adaptar o perfil para a estrutura do Domínio Puro
        const domainProfile = adaptSettingsProfileToDomain(activeProfile);

        // 3. Obter e adaptar Trades (apenas trades fechados interessam pro risco)
        const closedTrades = tradesStore.trades.filter(t => t.exit_price !== null && t.exit_price !== undefined);
        const domainTrades = closedTrades.map(adaptTradeToDomain);

        // 4. Estruturar Parâmetros de GrowthPhase
        let domainGrowthPhase = undefined;
        let startingCapital = undefined;

        if (activeProfile.growth_plan_enabled && activeProfile.growth_phases && activeProfile.growth_phases.length > activeProfile.current_phase_index) {
            const rawPhase = activeProfile.growth_phases[activeProfile.current_phase_index];
            domainGrowthPhase = adaptGrowthPhaseToDomain(rawPhase);

            // Determinar Capital Base (para drawdowns) baseado na regra da conta escolhida vs fixed capital
            if (activeProfile.capital_source === 'Fixed') {
                startingCapital = activeProfile.fixed_capital;
            } else if (activeProfile.linked_account_id) {
                const linkedAccount = settingsStore.accounts.find(a => a.id === activeProfile.linked_account_id);
                startingCapital = linkedAccount ? linkedAccount.balance : undefined;
            }
        }

        // 5. Acionar o Agregador Central Puro do Domínio
        return buildRiskCockpitState(
            domainProfile,
            domainTrades,
            domainGrowthPhase,
            startingCapital
        );
    }

    /**
     * Retorna o Perfil de Risco de Ativo resolvido para o ativo selecionado na UI.
     */
    get resolvedAssetRiskProfile() {
        const activeProfile = settingsStore.activeProfile;
        if (!activeProfile || !activeProfile.active || !this.activeAssetId) return null;

        const asset = settingsStore.assets.find(a => a.id === this.activeAssetId);
        if (!asset) return null;

        const linkedProfileIds = activeProfile.linked_asset_risk_profile_ids || [];
        const validProfiles = settingsStore.assetRiskProfiles.filter(ap => linkedProfileIds.includes(ap.id as string));
        
        return validProfiles.find(ap => ap.asset_id === asset.id) || null;
    }

    /**
     * Retorna o contexto completo de Growth Plan associado ao Perfil ativo e Ativo selecionado.
     * Segue a hierarquia de Prioridade (Override):
     * 1. AssetRiskProfile Override Phase
     * 2. Global RiskProfile Phase
     */
    get resolvedGrowthContext(): ResolvedGrowthContext | null {
        const activeProfile = settingsStore.activeProfile;
        if (!activeProfile || !activeProfile.active || !this.activeAssetId) return null;

        const asset = settingsStore.assets.find(a => a.id === this.activeAssetId);
        if (!asset) return null;

        const assetRiskProfile = this.resolvedAssetRiskProfile;
        
        // Exige vínculo explícito: sem perfil de ativo, sem avaliação de growth pra este ativo
        if (!assetRiskProfile) return null;

        // 1. Tenta resolver via AssetRiskProfile (Override prioritário)
        if (assetRiskProfile.growth_override_enabled && assetRiskProfile.growth_phases_override && assetRiskProfile.growth_phases_override.length > 0) {
            const currentObjPhaseIndex = assetRiskProfile.current_phase_index || 0;
            if (currentObjPhaseIndex < assetRiskProfile.growth_phases_override.length) {
                return {
                    asset,
                    assetRiskProfile,
                    riskProfile: activeProfile,
                    growthSourceType: "assetProfile",
                    growthPhase: assetRiskProfile.growth_phases_override[currentObjPhaseIndex]
                };
            }
        }

        // 2. Fallback para o Global RiskProfile
        if (activeProfile.growth_plan_enabled && activeProfile.growth_phases && activeProfile.growth_phases.length > 0) {
            const currentPhaseIndex = activeProfile.current_phase_index || 0;
            if (currentPhaseIndex < activeProfile.growth_phases.length) {
                return {
                    asset,
                    assetRiskProfile,
                    riskProfile: activeProfile,
                    growthSourceType: "global",
                    growthPhase: activeProfile.growth_phases[currentPhaseIndex]
                };
            }
        }

        // 3. Nenhum Contexto de Crescimento Aplicável
        return null;
    }

    /**
     * Motor de Position Sizing (Entrada Pura formatada pelo Adapter)
     */
    get positionSizingInput(): PositionSizingInput | null {
        const activeProfile = settingsStore.activeProfile;
        if (!activeProfile || !activeProfile.active || !this.activeAssetId) return null;

        const asset = settingsStore.assets.find(a => a.id === this.activeAssetId);
        if (!asset) return null;

        // ETAPA 3 & 4: Reutiliza logica do getter para obter o perfil resolvido na UI tbm.
        const assetRiskProfile = this.resolvedAssetRiskProfile;
        
        // Falha explícita se não existir perfil de ativo vinculado
        if (!assetRiskProfile) return null;

        let currentPhase = undefined;
        let dynamicBalance = undefined;

        if (activeProfile.growth_plan_enabled && activeProfile.growth_phases && activeProfile.growth_phases.length > activeProfile.current_phase_index) {
            currentPhase = activeProfile.growth_phases[activeProfile.current_phase_index];
        }

        if (activeProfile.capital_source === 'LinkedAccount' && activeProfile.linked_account_id) {
            const linkedAccount = settingsStore.accounts.find(a => a.id === activeProfile.linked_account_id);
            if (linkedAccount) {
                dynamicBalance = linkedAccount.balance;
            }
        }

        return adaptPositionSizingInput(
            activeProfile,
            asset,
            assetRiskProfile,
            currentPhase,
            dynamicBalance
        );
    }

    /**
     * Motor de Position Sizing (Resultado de Cálculo Puro do Motor Matemático)
     */
    get positionSizingResult(): PositionSizingResult | null {
        const input = this.positionSizingInput;
        if (!input) return null;
        return calculatePositionSizing(input);
    }
}

export const riskStore = new RiskStore();

// forced-republish-cache-breaker: 2026-03-17
