<script lang="ts">
  import { Button } from "$lib/components/ui/button";
</script>

<div class="flex h-screen w-full items-center justify-center bg-background">
  <div class="text-center">
    <h1 class="text-4xl font-bold tracking-tight text-primary mb-4">
      TraderLog Pro
    </h1>
    <p class="text-muted-foreground mb-8">
      A evolução do seu diário de trading.
    </p>
    <div class="flex gap-4 justify-center">
      <Button variant="default">Novo Trade</Button>
      <Button variant="outline">Ver Dashboard</Button>
    </div>
  </div>
</div>
