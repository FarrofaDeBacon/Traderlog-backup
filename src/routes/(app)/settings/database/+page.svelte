<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { RefreshCw, Search, Database, Trash2 } from "lucide-svelte";
    import { toast } from "svelte-sonner";

    let isChecking = $state(false);
    let statusMessage = $state("");

    async function checkDatabase() {
        isChecking = true;
        statusMessage = "";
        try {
            console.log("Checking database status...");
            const status = await invoke<string>("check_database_status");
            console.log("Database status result:", status);
            statusMessage = status;
            toast.success("Status verificado com sucesso!");
        } catch (e) {
            console.error("Error checking database status:", e);
            const errorMsg =
                typeof e === "string"
                    ? e
                    : (e as Error).message ||
                      "Erro desconhecido ao verificar status";
            statusMessage = "ERRO: " + errorMsg;
            toast.error("Erro ao verificar: " + errorMsg);
        } finally {
            isChecking = false;
        }
    }
    let isRestoring = $state(false);

    async function handleRestoreDefaults() {
        if (
            !confirm(
                "Isso irá verificar e corrigir dados padrão faltantes (Moedas, Tipos de Ativo, etc.), sem apagar seus dados. Deseja continuar?",
            )
        ) {
            return;
        }

        isRestoring = true;
        try {
            await invoke("ensure_defaults");
            toast.success(
                "Dados padrão verificados e restaurados com sucesso!",
            );
            setTimeout(() => window.location.reload(), 1500);
        } catch (e) {
            toast.error("Erro ao restaurar: " + (e as Error).message);
        } finally {
            isRestoring = false;
        }
    }

    // Demo Data Management
    let demoAccounts = $state([
        {
            id: "account:demo_forex",
            name: "Conta FOREX",
            trades: "60 trades",
            checked: false,
        },
        {
            id: "account:demo_b3_acoes",
            name: "Conta B3 - Ações",
            trades: "55 trades",
            checked: false,
        },
        {
            id: "account:demo_b3_futuros",
            name: "Conta B3 - Futuros",
            trades: "70 trades",
            checked: false,
        },
        {
            id: "account:demo_nasdaq",
            name: "Conta NASDAQ",
            trades: "60 trades",
            checked: false,
        },
        {
            id: "account:demo_crypto",
            name: "Conta Crypto (Binance)",
            trades: "65 trades",
            checked: false,
        },
    ]);

    let isGenerating = $state(false);
    let isDeleting = $state(false);
    let isCleaning = $state(false);

    async function handleCleanAll() {
        isCleaning = true;
        try {
            await invoke("delete_all_demo_trades");
            toast.success("Todos os trades demo foram removidos!");
        } catch (e) {
            toast.error("Erro ao limpar: " + (e as Error).message);
        } finally {
            isCleaning = false;
        }
    }

    async function handleGenerateDemo() {
        const selected = demoAccounts.filter((acc) => acc.checked);
        if (selected.length === 0) {
            toast.error("Selecione pelo menos uma conta");
            return;
        }

        isGenerating = true;
        try {
            for (const account of selected) {
                await invoke("seed_demo_account", { accountId: account.id });
            }
            toast.success(`${selected.length} conta(s) populadas com sucesso!`);
            setTimeout(() => window.location.reload(), 1500);
        } catch (error) {
            console.error("Erro ao gerar demo:", error);
            // Show more detailed error if available
            const errorMsg =
                typeof error === "string"
                    ? error
                    : JSON.stringify(error, null, 2);
            toast.error(`Erro ao gerar dados: ${errorMsg}`);
        } finally {
            isGenerating = false;
        }
    }

    async function handleDeleteDemo() {
        const selected = demoAccounts.filter((acc) => acc.checked);
        if (selected.length === 0) {
            toast.error("Selecione pelo menos uma conta");
            return;
        }

        if (
            !confirm(
                `Tem certeza que deseja apagar os dados de ${selected.length} conta(s)? Esta ação não pode ser desfeita.`,
            )
        ) {
            return;
        }

        isDeleting = true;
        try {
            for (const account of selected) {
                await invoke("delete_demo_account_data", {
                    accountId: account.id,
                });
            }
            toast.success(`Dados de ${selected.length} conta(s) removidos!`);
            setTimeout(() => window.location.reload(), 1500);
        } catch (e) {
            toast.error("Erro ao apagar dados: " + (e as Error).message);
        } finally {
            isDeleting = false;
        }
    }
    async function handleForceReseed() {
        if (
            !confirm(
                "ATENÇÃO: Isso apagará TODOS os dados (Trades, Contas, Configurações) e recriará o banco do zero com dados de exemplo. Tem certeza absoluta?",
            )
        ) {
            return;
        }

        isResetting = true;
        try {
            await invoke("force_reseed");
            toast.success("Banco de dados resetado e populado com sucesso!");
            setTimeout(() => window.location.reload(), 1500);
        } catch (e) {
            const errorMsg =
                typeof e === "string"
                    ? e
                    : (e as Error).message || "Erro desconhecido";
            toast.error("Erro ao resetar: " + errorMsg);
        } finally {
            isResetting = false;
        }
    }

    let isResetting = $state(false);
</script>

<div class="container mx-auto p-6 space-y-6">
    <div>
        <h1 class="text-3xl font-bold">Gerenciar Banco de Dados</h1>
        <p class="text-muted-foreground mt-2">
            Ferramentas para diagnosticar e gerenciar o banco de dados.
        </p>
    </div>

    <Card.Root>
        <Card.Header>
            <div class="flex items-center gap-2">
                <Search class="w-5 h-5 text-blue-500" />
                <Card.Title>Verificar Status do Banco</Card.Title>
            </div>
            <Card.Description>
                Veja quantos registros existem em cada tabela principal.
            </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
            <Button
                onclick={checkDatabase}
                disabled={isChecking}
                variant="outline"
            >
                {#if isChecking}
                    <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                    Verificando...
                {:else}
                    <Search class="w-4 h-4 mr-2" />
                    Verificar Status
                {/if}
            </Button>

            {#if statusMessage}
                <div
                    class="rounded-lg overflow-hidden border {statusMessage.startsWith(
                        'ERRO',
                    )
                        ? 'border-destructive/30'
                        : 'border-blue-200 dark:border-blue-900/50'}"
                >
                    <div
                        class="{statusMessage.startsWith('ERRO')
                            ? 'bg-destructive/10'
                            : 'bg-blue-500/5'} p-4"
                    >
                        <pre
                            class="text-sm whitespace-pre-wrap font-mono {statusMessage.startsWith(
                                'ERRO',
                            )
                                ? 'text-destructive'
                                : 'text-blue-600 dark:text-blue-400'}">{statusMessage}</pre>
                    </div>
                </div>
            {/if}
        </Card.Content>
    </Card.Root>

    <Card.Root>
        <Card.Header>
            <div class="flex items-center gap-2">
                <RefreshCw class="w-5 h-5 text-emerald-500" />
                <Card.Title>Restaurar Padrões do Sistema</Card.Title>
            </div>
            <Card.Description>
                Use esta opção caso note que opções como "Tipos de Ativo" ou
                "Estados Emocionais" estão faltando.
            </Card.Description>
        </Card.Header>
        <Card.Content>
            <div
                class="bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-900 rounded p-4 mb-4"
            >
                <p class="text-sm text-emerald-800 dark:text-emerald-200">
                    <strong>Seguro para uso:</strong> Esta ação verifica a
                    integridade dos dados essenciais do sistema e recria apenas
                    o que estiver faltando ou incorreto.
                    <strong>Seus trades e contas NÃO serão apagados.</strong>
                </p>
            </div>

            <Button
                onclick={handleRestoreDefaults}
                disabled={isRestoring}
                class="bg-emerald-600 hover:bg-emerald-700 text-white"
            >
                {#if isRestoring}
                    <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                    Restaurando...
                {:else}
                    <RefreshCw class="w-4 h-4 mr-2" />
                    Verificar e Restaurar Padrões
                {/if}
            </Button>
        </Card.Content>
    </Card.Root>

    <!-- Demo Data Management -->
    <Card.Root>
        <Card.Header>
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <Database class="w-5 h-5 text-purple-500" />
                    <Card.Title>Dados de Demonstração</Card.Title>
                </div>
                <Button
                    variant="outline"
                    size="sm"
                    onclick={handleCleanAll}
                    disabled={isCleaning}
                    class="text-xs border-purple-500/30 hover:bg-purple-500/10 text-purple-500"
                >
                    {#if isCleaning}
                        <RefreshCw class="w-3 h-3 mr-2 animate-spin" />
                        Limpando...
                    {:else}
                        <Trash2 class="w-3 h-3 mr-2" />
                        Limpar Tudo (Reset)
                    {/if}
                </Button>
            </div>
            <Card.Description>
                Gere ou remova trades de exemplo para testar filtros e
                funcionalidades.
            </Card.Description>
        </Card.Header>
        <Card.Content class="space-y-6">
            <!-- Generate Section -->
            <div class="space-y-4">
                <h3 class="font-semibold text-sm">Gerar Dados</h3>
                <div class="space-y-3 border rounded-lg p-4 bg-muted/30">
                    {#each demoAccounts as account}
                        <label
                            class="flex items-center gap-3 cursor-pointer hover:bg-accent/50 p-2 rounded transition-colors"
                        >
                            <Checkbox bind:checked={account.checked} />
                            <div class="flex-1">
                                <p class="font-medium">{account.name}</p>
                                <p class="text-sm text-muted-foreground">
                                    {account.trades}
                                </p>
                            </div>
                        </label>
                    {/each}
                </div>
                <Button
                    onclick={handleGenerateDemo}
                    disabled={isGenerating}
                    class="w-full bg-purple-600 hover:bg-purple-700 text-white"
                >
                    {#if isGenerating}
                        <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                        Gerando...
                    {:else}
                        <Database class="w-4 h-4 mr-2" />
                        Gerar Selecionados
                    {/if}
                </Button>
            </div>

            <!-- Delete Section -->
            <div class="space-y-4 border-t pt-6">
                <h3 class="font-semibold text-sm text-destructive">
                    Apagar Dados
                </h3>
                <div
                    class="bg-destructive/10 border border-destructive/30 rounded p-3"
                >
                    <p class="text-sm text-destructive">
                        <strong>Atenção:</strong> Esta ação removerá permanentemente
                        os trades das contas selecionadas.
                    </p>
                </div>
                <AlertDialog.Root>
                    <AlertDialog.Trigger>
                        {#snippet child({
                            props,
                        }: {
                            props: Record<string, any>;
                        })}
                            <Button
                                {...props}
                                disabled={isDeleting}
                                variant="destructive"
                                class="w-full"
                            >
                                {#if isDeleting}
                                    <RefreshCw
                                        class="w-4 h-4 mr-2 animate-spin"
                                    />
                                    Apagando...
                                {:else}
                                    <Trash2 class="w-4 h-4 mr-2" />
                                    Apagar Selecionados
                                {/if}
                            </Button>
                        {/snippet}
                    </AlertDialog.Trigger>
                    <AlertDialog.Content>
                        <AlertDialog.Header>
                            <AlertDialog.Title
                                >Confirmar Exclusão</AlertDialog.Title
                            >
                            <AlertDialog.Description>
                                Tem certeza que deseja apagar os dados de
                                demonstração das contas selecionadas? Esta ação
                                não pode ser desfeita.
                            </AlertDialog.Description>
                        </AlertDialog.Header>
                        <AlertDialog.Footer>
                            <AlertDialog.Cancel>Cancelar</AlertDialog.Cancel>
                            <AlertDialog.Action
                                onclick={handleDeleteDemo}
                                class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                            >
                                Sim, Apagar
                            </AlertDialog.Action>
                        </AlertDialog.Footer>
                    </AlertDialog.Content>
                </AlertDialog.Root>
            </div>
        </Card.Content>
    </Card.Root>

    <!-- Danger Zone: Full Reset -->
    <Card.Root class="border-destructive/50">
        <Card.Header>
            <div class="flex items-center gap-2">
                <Trash2 class="w-5 h-5 text-destructive" />
                <Card.Title class="text-destructive"
                    >Zona de Perigo: Reset Total</Card.Title
                >
            </div>
            <Card.Description>
                Apaga todos os dados e recria o banco do zero com dados de
                fábrica.
            </Card.Description>
        </Card.Header>
        <Card.Content>
            <div
                class="bg-destructive/10 border border-destructive/30 rounded p-4 mb-4"
            >
                <p class="text-sm text-destructive font-bold">
                    CUIDADO: Todos os seus trades, contas e personalizações
                    serão perdidos.
                </p>
            </div>
            <Button
                onclick={handleForceReseed}
                disabled={isResetting}
                variant="destructive"
                class="w-full"
            >
                {#if isResetting}
                    <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                    Resetando...
                {:else}
                    <Trash2 class="w-4 h-4 mr-2" />
                    Resetar Banco de Dados (Completo)
                {/if}
            </Button>
        </Card.Content>
    </Card.Root>
</div>
