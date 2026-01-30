<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import * as Select from "$lib/components/ui/select";
    import { Separator } from "$lib/components/ui/separator";
    import ModeToggle from "$lib/components/mode-toggle/ModeToggle.svelte";

    // Mock Data
    const strategies = [
        { value: "scalping", label: "Scalping" },
        { value: "daytrade", label: "Day Trade" },
        { value: "swing", label: "Swing Trade" },
    ];
</script>

<div class="space-y-6">
    <div class="space-y-0.5">
        <h2 class="text-2xl font-bold tracking-tight">Configurações</h2>
        <p class="text-muted-foreground">
            Gerencie suas preferências de conta e do aplicativo.
        </p>
    </div>
    <Separator class="my-6" />

    <Tabs.Root value="general" class="space-y-4">
        <Tabs.List>
            <Tabs.Trigger value="general">Geral</Tabs.Trigger>
            <Tabs.Trigger value="appearance">Aparência</Tabs.Trigger>
            <Tabs.Trigger value="profile">Perfil</Tabs.Trigger>
            <Tabs.Trigger value="backup">Backup</Tabs.Trigger>
        </Tabs.List>

        <!-- General Settings -->
        <Tabs.Content value="general" class="space-y-4">
            <Card.Root>
                <Card.Header>
                    <Card.Title>Preferências de Trading</Card.Title>
                    <Card.Description
                        >Configure como seus trades são registrados por padrão.</Card.Description
                    >
                </Card.Header>
                <Card.Content class="space-y-4">
                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2">
                            <Label>Estratégia Padrão</Label>
                            <Select.Root>
                                <Select.Trigger class="w-full">
                                    <Select.Value placeholder="Selecione..." />
                                </Select.Trigger>
                                <Select.Content>
                                    {#each strategies as strategy}
                                        <Select.Item value={strategy.value}
                                            >{strategy.label}</Select.Item
                                        >
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        </div>
                        <div class="space-y-2">
                            <Label>Risco Padrão (%)</Label>
                            <Input type="number" placeholder="1.0" />
                        </div>
                    </div>
                    <div class="flex items-center space-x-2">
                        <Switch id="auto-calc" />
                        <Label htmlFor="auto-calc"
                            >Calcular Tamanho da Posição Automaticamente</Label
                        >
                    </div>
                </Card.Content>
            </Card.Root>
        </Tabs.Content>

        <!-- Appearance Settings -->
        <Tabs.Content value="appearance" class="space-y-4">
            <Card.Root>
                <Card.Header>
                    <Card.Title>Tema & Visual</Card.Title>
                    <Card.Description
                        >Personalize a aparência do TraderLog Pro.</Card.Description
                    >
                </Card.Header>
                <Card.Content class="space-y-4">
                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-base">Tema do Aplicativo</Label>
                            <p class="text-sm text-muted-foreground">
                                Alternar entre modo claro e escuro.
                            </p>
                        </div>
                        <ModeToggle />
                    </div>
                    <Separator />
                    <div class="flex items-center space-x-2">
                        <Switch id="compact-mode" />
                        <Label htmlFor="compact-mode"
                            >Modo Compacto (Listas mais densas)</Label
                        >
                    </div>
                </Card.Content>
            </Card.Root>
        </Tabs.Content>

        <!-- Profile Settings -->
        <Tabs.Content value="profile" class="space-y-4">
            <Card.Root>
                <Card.Header>
                    <Card.Title>Meu Perfil</Card.Title>
                    <Card.Description
                        >Suas informações de trader.</Card.Description
                    >
                </Card.Header>
                <Card.Content class="space-y-4">
                    <div class="grid gap-2">
                        <Label for="name">Nome de Exibição</Label>
                        <Input
                            id="name"
                            placeholder="Seu nome"
                            defaultValue="Farrofa DeBacon"
                        />
                    </div>
                    <div class="grid gap-2">
                        <Label for="bio">Bio</Label>
                        <Input
                            id="bio"
                            placeholder="Sua filosofia de trading..."
                        />
                    </div>
                </Card.Content>
                <Card.Footer>
                    <Button>Salvar Alterações</Button>
                </Card.Footer>
            </Card.Root>
        </Tabs.Content>

        <!-- Backup & Data -->
        <Tabs.Content value="backup" class="space-y-4">
            <Card.Root>
                <Card.Header>
                    <Card.Title>Dados & Backup</Card.Title>
                    <Card.Description
                        >Gerencie seus dados locais.</Card.Description
                    >
                </Card.Header>
                <Card.Content class="space-y-4">
                    <div class="flex gap-4">
                        <Button variant="outline">Exportar Tudo (JSON)</Button>
                        <Button variant="destructive"
                            >Limpar Banco de Dados</Button
                        >
                    </div>
                </Card.Content>
            </Card.Root>
        </Tabs.Content>
    </Tabs.Root>
</div>
