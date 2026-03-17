import { settingsStore } from './settings.svelte';
import { tradesStore } from './trades.svelte';
import { buildRiskCockpitState, type RiskCockpitState } from '$lib/domain/risk';
import { 
    adaptSettingsProfileToDomain, 
    adaptTradeToDomain, 
    adaptGrowthPhaseToDomain 
} from '$lib/domain/risk/risk-adapters';

/**
 * Store reativo estritamente focado no Cockpit de Risco.
 * Consome os stores de `settings` e `trades`, converte usando `risk-adapters`
 * e expõe o resultado do domínio em tempo real via getter derivado.
 */
class RiskStore {
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
}

export const riskStore = new RiskStore();
