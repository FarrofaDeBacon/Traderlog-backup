<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
    import { Input } from "$lib/components/ui/input";
    import { Search, User, LogOut, Settings } from "lucide-svelte";
    import * as Avatar from "$lib/components/ui/avatar";
    import ModeToggle from "$lib/components/mode-toggle/ModeToggle.svelte";

    // Props for mobile trigger injection if needed, but Sidebar handles its own trigger currently.
    // We might want to move the trigger here if we want a unified header.
    // For now, let's keep the header focused on Search + User.
    // The Sidebar component will be placed alongside this in the layout.
</script>

<header
    class="flex h-14 items-center gap-4 border-b bg-muted/40 px-4 lg:h-[60px] lg:px-6"
>
    <!-- Mobile Menu Trigger Area (Occupied by Sidebar Trigger in Layout) -->
    <div class="w-full flex-1">
        <form>
            <div class="relative">
                <Search
                    class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
                />
                <Input
                    type="search"
                    placeholder="Buscar trades, notas..."
                    class="w-full appearance-none bg-background pl-8 shadow-none md:w-2/3 lg:w-1/3"
                />
            </div>
        </form>
    </div>
    <ModeToggle />
    <DropdownMenu.Root>
        <DropdownMenu.Trigger>
            {#snippet child({ props }: { props: Record<string, any> })}
                <Button
                    variant="ghost"
                    size="icon"
                    class="rounded-full"
                    {...props}
                >
                    <Avatar.Root class="h-8 w-8">
                        <Avatar.Image src="" alt="User" />
                        <Avatar.Fallback>FD</Avatar.Fallback>
                    </Avatar.Root>
                    <span class="sr-only">Toggle user menu</span>
                </Button>
            {/snippet}
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="end">
            <DropdownMenu.Label>Minha Conta</DropdownMenu.Label>
            <DropdownMenu.Separator />
            <DropdownMenu.Item>Configurações</DropdownMenu.Item>
            <DropdownMenu.Item>Suporte</DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item>Sair</DropdownMenu.Item>
        </DropdownMenu.Content>
    </DropdownMenu.Root>
</header>
