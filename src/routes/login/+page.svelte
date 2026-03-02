<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import { Separator } from "$lib/components/ui/separator";
    import { Mail, Lock, LogIn, Github, Chrome } from "lucide-svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";

    import { invoke } from "@tauri-apps/api/core";

    let email = $state("admin@traderlog.pro");
    let password = $state("123456");
    let isLoading = $state(false);
    let showRecovery = $state(false);
    let recoveryData = $state({
        key: "",
        newPassword: "",
        confirmPassword: "",
    });

    async function handleLogin() {
        if (!password) {
            toast.error($t("auth.login.error.fillAll"));
            return;
        }

        isLoading = true;

        try {
            const success = await settingsStore.login(email, password);
            if (success) {
                toast.success($t("auth.login.success"));
                // Force reload to ensure clean state and session persistence
                window.location.href = "/";
            } else {
                toast.error($t("auth.login.error.invalidCredentials"));
                isLoading = false;
            }
        } catch (e) {
            console.error("Login component error:", e);
            toast.error($t("general.error"));
            isLoading = false;
        }
    }

    async function handleResetPassword() {
        if (
            !recoveryData.key ||
            !recoveryData.newPassword ||
            !recoveryData.confirmPassword
        ) {
            toast.error($t("auth.login.recovery.error.fillAll"));
            return;
        }

        if (recoveryData.newPassword !== recoveryData.confirmPassword) {
            toast.error($t("auth.login.recovery.error.mismatch"));
            return;
        }

        isLoading = true;
        try {
            const success = await invoke("reset_password", {
                recoveryKey: recoveryData.key,
                newPassword: recoveryData.newPassword,
            });

            if (success) {
                toast.success($t("auth.login.recovery.success"));
                showRecovery = false;
                password = recoveryData.newPassword;
            } else {
                toast.error($t("auth.login.recovery.error.invalidKey"));
            }
        } catch (e) {
            console.error("Reset failed:", e);
            toast.error($t("auth.login.recovery.error.generic") + ": " + e);
        } finally {
            isLoading = false;
        }
    }
</script>

<div
    class="min-h-screen flex items-center justify-center bg-background p-4 relative overflow-hidden text-foreground"
>
    <!-- Ambient Background Effects -->
    <div
        class="absolute top-[-10%] left-[-10%] w-[500px] h-[500px] bg-primary/20 rounded-full blur-[128px] pointer-events-none opacity-50 dark:opacity-30"
    ></div>
    <div
        class="absolute bottom-[-10%] right-[-10%] w-[500px] h-[500px] bg-blue-500/10 rounded-full blur-[128px] pointer-events-none opacity-50 dark:opacity-20"
    ></div>

    {#if !showRecovery}
        <Card.Root
            class="w-full max-w-md border-border bg-card/50 backdrop-blur-xl shadow-2xl relative z-10"
        >
            <Card.Header class="space-y-1 text-center pb-8 pt-10">
                <div
                    class="mx-auto w-12 h-12 bg-primary/10 rounded-xl flex items-center justify-center mb-4 border border-primary/20"
                >
                    <LogIn class="w-6 h-6 text-primary" />
                </div>
                <Card.Title
                    class="text-2xl font-bold tracking-tight text-foreground"
                    >{$t("auth.login.title")}</Card.Title
                >
                <Card.Description class="text-muted-foreground">
                    {$t("auth.login.description")}
                </Card.Description>
            </Card.Header>
            <Card.Content class="space-y-4">
                <div class="space-y-2">
                    <Label for="email" class="text-foreground/80"
                        >{$t("auth.login.email")}</Label
                    >
                    <div class="relative">
                        <Mail
                            class="absolute left-3 top-3 h-4 w-4 text-muted-foreground"
                        />
                        <Input
                            id="email"
                            type="email"
                            placeholder="nome@exemplo.com"
                            class="pl-9 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                            bind:value={email}
                        />
                    </div>
                </div>
                <div class="space-y-2">
                    <div class="flex items-center justify-between">
                        <Label for="password" class="text-foreground/80"
                            >{$t("auth.login.password")}</Label
                        >
                        <Button
                            variant="link"
                            class="h-auto p-0 text-xs text-primary hover:no-underline"
                            onclick={() => (showRecovery = true)}
                            >{$t("auth.login.forgotPassword")}</Button
                        >
                    </div>
                    <div class="relative">
                        <Lock
                            class="absolute left-3 top-3 h-4 w-4 text-muted-foreground"
                        />
                        <Input
                            id="password"
                            type="password"
                            placeholder="••••••••"
                            class="pl-9 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                            bind:value={password}
                            onkeydown={(e) =>
                                e.key === "Enter" && handleLogin()}
                        />
                    </div>
                </div>
                <Button
                    class="w-full mt-4 bg-primary hover:bg-primary/90 text-primary-foreground font-semibold shadow-lg shadow-primary/20 h-11"
                    onclick={handleLogin}
                    disabled={isLoading}
                >
                    {#if isLoading}
                        {$t("auth.login.loading")}
                    {:else}
                        <div class="flex items-center gap-2">
                            {$t("auth.login.submit")}
                        </div>
                    {/if}
                </Button>
            </Card.Content>
        </Card.Root>
    {:else}
        <Card.Root
            class="w-full max-w-md border-border bg-card/50 backdrop-blur-xl shadow-2xl relative z-10"
        >
            <Card.Header class="space-y-1 text-center pb-8 pt-10">
                <div
                    class="mx-auto w-12 h-12 bg-primary/10 rounded-xl flex items-center justify-center mb-4 border border-primary/20"
                >
                    <Lock class="w-6 h-6 text-primary" />
                </div>
                <Card.Title
                    class="text-2xl font-bold tracking-tight text-foreground"
                >
                    {$t("auth.login.recovery.title")}
                </Card.Title>
                <Card.Description class="text-muted-foreground">
                    {$t("auth.login.recovery.description")}
                </Card.Description>
            </Card.Header>
            <Card.Content class="space-y-4">
                <div class="space-y-2">
                    <Label for="recoveryKey" class="text-foreground/80"
                        >{$t("auth.login.recovery.keyLabel")}</Label
                    >
                    <Input
                        id="recoveryKey"
                        type="text"
                        placeholder="XXXX-XXXX-XXXX-XXXX"
                        class="bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                        bind:value={recoveryData.key}
                    />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-2">
                        <Label for="newPassword" class="text-foreground/80"
                            >{$t("auth.login.recovery.newPassword")}</Label
                        >
                        <Input
                            id="newPassword"
                            type="password"
                            placeholder="••••••••"
                            class="bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                            bind:value={recoveryData.newPassword}
                        />
                    </div>
                    <div class="space-y-2">
                        <Label
                            for="confirmNewPassword"
                            class="text-foreground/80"
                            >{$t("auth.login.recovery.confirm")}</Label
                        >
                        <Input
                            id="confirmNewPassword"
                            type="password"
                            placeholder="••••••••"
                            class="bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                            bind:value={recoveryData.confirmPassword}
                        />
                    </div>
                </div>
                <Button
                    class="w-full font-bold h-11"
                    onclick={handleResetPassword}
                    disabled={isLoading}
                >
                    {#if isLoading}
                        {$t("auth.login.recovery.loading")}
                    {:else}
                        {$t("auth.login.recovery.submit")}
                    {/if}
                </Button>
                <Button
                    variant="ghost"
                    class="w-full text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                    onclick={() => (showRecovery = false)}
                >
                    {$t("auth.login.recovery.back")}
                </Button>
            </Card.Content>
        </Card.Root>
    {/if}
</div>
