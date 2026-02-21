<script lang="ts">
    import { t } from "svelte-i18n";
    import * as Card from "$lib/components/ui/card";
    import { Hammer } from "lucide-svelte";
</script>

<div
    class="flex flex-col items-center justify-center h-[60vh] text-center space-y-4"
>
    <div class="p-6 bg-muted rounded-full">
        <Hammer class="w-12 h-12 text-muted-foreground" />
    </div>
    <h2 class="text-2xl font-bold tracking-tight">
        Módulo Fiscal em Construção
    </h2>
    <p class="text-muted-foreground max-w-[500px]">
        Estamos implementando o dashboard de cálculo de imposto (DARF). Por
        enquanto, configure suas regras fiscais no menu de Configurações.
    </p>
    <div class="flex gap-4">
        <a
            href="/settings/fiscal"
            class="text-primary hover:underline font-medium"
        >
            Ir para Configurações Fiscais &rarr;
        </a>
    </div>
</div>
