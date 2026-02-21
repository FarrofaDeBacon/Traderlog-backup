export interface OperationalState {
    preTradeChecklistDone: boolean;
    lastMentalCheck: Date | null;
    isBlocked: boolean;
    blockReason: string;
}

class OperationalManagement {
    state = $state<OperationalState>({
        preTradeChecklistDone: false,
        lastMentalCheck: null,
        isBlocked: false,
        blockReason: ""
    });

    canTrade = $derived(!this.state.isBlocked && this.state.preTradeChecklistDone);

    resetChecklist() {
        this.state.preTradeChecklistDone = false;
    }

    completeChecklist() {
        this.state.preTradeChecklistDone = true;
    }

    block(reason: string) {
        this.state.isBlocked = true;
        this.state.blockReason = reason;
    }

    unblock() {
        this.state.isBlocked = false;
        this.state.blockReason = "";
    }
}

export const operationalManagement = new OperationalManagement();
