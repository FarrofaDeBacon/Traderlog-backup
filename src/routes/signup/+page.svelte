<script lang="ts">
    import { onMount } from "svelte";
    onMount(() => {
        console.log("[Register] Page loaded");
    });
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Separator } from "$lib/components/ui/separator";
    import * as Card from "$lib/components/ui/card";
    import {
        Mail,
        Lock,
        User,
        IdCard,
        Calendar,
        ArrowRight,
    } from "lucide-svelte";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { goto } from "$app/navigation";
    import { toast } from "svelte-sonner";
    import { t } from "svelte-i18n";
    import { invoke } from "@tauri-apps/api/core";
    import { fade, fly } from "svelte/transition";

    let formData = $state({
        name: "",
        email: "",
        cpf: "",
        birthDate: "",
        password: "",
        confirmPassword: "",
    });

    let isLoading = $state(false);
    let recoveryKey = $state("");
    let showRecovery = $state(false);

    function generateKey() {
        // Simple grouped alphanumeric key
        const chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let result = "";
        for (let i = 0; i < 16; i++) {
            if (i > 0 && i % 4 === 0) result += "-";
            result += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return result;
    }

    function formatCPF(v: string) {
        v = v.replace(/\D/g, "");
        if (v.length > 11) v = v.substring(0, 11);
        return v.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, "$1.$2.$3-$4");
    }

    function handleCpfInput(e: any) {
        formData.cpf = formatCPF(e.target.value);
    }

    async function handleRegister() {
        if (
            !formData.name ||
            !formData.email ||
            !formData.cpf ||
            !formData.password
        ) {
            toast.error($t("auth.register.error.fillAll"));
            return;
        }

        if (formData.password !== formData.confirmPassword) {
            toast.error($t("auth.register.error.passwordsMismatch"));
            return;
        }

        isLoading = true;
        recoveryKey = generateKey();

        try {
            // Save the profile using the standard UPSERT logic in backend
            await invoke("save_user_profile", {
                profile: {
                    ...settingsStore.userProfile,
                    name: formData.name,
                    email: formData.email,
                    cpf: formData.cpf,
                    birth_date: formData.birthDate,
                    password_hash: formData.password, // Plain text here, backend will hash it
                    recovery_hash: recoveryKey, // Backend will hash it
                    onboarding_completed: false, // Force onboarding to start
                },
            });

            // Update local store partially
            settingsStore.userProfile.name = formData.name;
            settingsStore.userProfile.email = formData.email;
            settingsStore.userProfile.cpf = formData.cpf;
            settingsStore.userProfile.birth_date = formData.birthDate;

            // Log in user immediately
            localStorage.setItem("isLoggedIn", "true");
            settingsStore.isLoggedIn = true;

            toast.success($t("auth.register.success"));
            showRecovery = true;
        } catch (error) {
            console.error("Registration failed:", error);
            toast.error("Erro ao criar conta: " + error);
        } finally {
            isLoading = false;
        }
    }

    function confirmRecovery() {
        window.location.href = "/";
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
        class="absolute bottom-[-10%] right-[-10%] w-[500px] h-[500px] bg-emerald-500/10 rounded-full blur-[128px] pointer-events-none opacity-50 dark:opacity-20"
    ></div>

    <div
        class="w-full max-w-lg relative z-10"
        in:fly={{ y: 20, duration: 600 }}
    >
        {#if !showRecovery}
            <div
                class="relative space-y-6 w-full max-w-md"
                in:fly={{ y: 20, duration: 800 }}
            >
                <!-- Form Header -->
                <div class="space-y-2 text-center">
                    <h1
                        class="text-3xl font-black tracking-tighter text-foreground"
                    >
                        {$t("auth.register.title")}
                    </h1>
                    <p class="text-muted-foreground">
                        {$t("auth.register.description")}
                    </p>
                </div>

                <Card.Root
                    class="border-border bg-card/50 backdrop-blur-xl shadow-2xl"
                >
                    <Card.Content class="pt-6 space-y-4">
                        <div class="grid gap-4">
                            <!-- Personal Info Section -->
                            <div class="grid gap-2">
                                <Label
                                    for="name"
                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
                                >
                                    {$t("auth.register.name")}
                                </Label>
                                <div class="relative">
                                    <User
                                        class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground"
                                    />
                                    <Input
                                        id="name"
                                        type="text"
                                        bind:value={formData.name}
                                        placeholder="John Doe"
                                        class="pl-10 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                                    />
                                </div>
                            </div>

                            <div class="grid grid-cols-2 gap-4">
                                <div class="grid gap-2">
                                    <Label
                                        for="cpf"
                                        class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
                                    >
                                        {$t("auth.register.cpf")}
                                    </Label>
                                    <div class="relative">
                                        <IdCard
                                            class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground"
                                        />
                                        <Input
                                            id="cpf"
                                            type="text"
                                            value={formData.cpf}
                                            oninput={handleCpfInput}
                                            placeholder="000.000.000-00"
                                            class="pl-10 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                                        />
                                    </div>
                                </div>
                                <div class="grid gap-2">
                                    <Label
                                        for="birthDate"
                                        class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
                                    >
                                        {$t("auth.register.birthDate")}
                                    </Label>
                                    <div class="relative">
                                        <Calendar
                                            class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground"
                                        />
                                        <Input
                                            id="birthDate"
                                            type="date"
                                            bind:value={formData.birthDate}
                                            class="pl-10 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                                        />
                                    </div>
                                </div>
                            </div>

                            <div class="grid gap-2">
                                <Label
                                    for="email"
                                    class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
                                >
                                    {$t("auth.register.email")}
                                </Label>
                                <div class="relative">
                                    <Mail
                                        class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground"
                                    />
                                    <Input
                                        id="email"
                                        type="email"
                                        bind:value={formData.email}
                                        placeholder="email@example.com"
                                        class="pl-10 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                                    />
                                </div>
                            </div>

                            <Separator class="bg-zinc-800 my-2" />

                            <!-- Password Section -->
                            <div class="grid grid-cols-2 gap-4">
                                <div class="grid gap-2">
                                    <Label
                                        for="password"
                                        class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
                                    >
                                        {$t("auth.register.password")}
                                    </Label>
                                    <div class="relative">
                                        <Lock
                                            class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground"
                                        />
                                        <Input
                                            id="password"
                                            type="password"
                                            bind:value={formData.password}
                                            placeholder="••••••••"
                                            class="pl-10 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                                        />
                                    </div>
                                </div>
                                <div class="grid gap-2">
                                    <Label
                                        for="confirmPassword"
                                        class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
                                    >
                                        {$t("auth.register.confirmPassword")}
                                    </Label>
                                    <div class="relative">
                                        <Lock
                                            class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground"
                                        />
                                        <Input
                                            id="confirmPassword"
                                            type="password"
                                            bind:value={
                                                formData.confirmPassword
                                            }
                                            placeholder="••••••••"
                                            class="pl-10 bg-muted/30 border-border text-foreground placeholder:text-muted-foreground/50 focus-visible:ring-primary/50"
                                            onkeydown={(e) =>
                                                e.key === "Enter" &&
                                                handleRegister()}
                                        />
                                    </div>
                                </div>
                            </div>

                            <Button
                                class="w-full mt-4 font-black shadow-lg shadow-primary/20 h-11"
                                onclick={handleRegister}
                                disabled={isLoading}
                            >
                                {#if isLoading}
                                    <span class="animate-pulse"
                                        >{$t("auth.register.loading")}</span
                                    >
                                {:else}
                                    {$t("auth.register.submit")}
                                    <ArrowRight class="ml-2 h-4 w-4" />
                                {/if}
                            </Button>
                        </div>
                    </Card.Content>
                    <Card.Footer
                        class="flex flex-col gap-4 border-t border-border/20 pt-6"
                    >
                        <Button
                            variant="link"
                            class="text-muted-foreground hover:text-primary transition-colors"
                            onclick={() => goto("/login")}
                        >
                            {$t("auth.register.alreadyHaveAccount")}
                            {$t("auth.register.login")}
                        </Button>
                    </Card.Footer>
                </Card.Root>

                <!-- Terms -->
                <p
                    class="text-[10px] text-center text-muted-foreground/60 px-8 uppercase tracking-widest leading-relaxed"
                >
                    {$t("auth.register.terms")}
                </p>
            </div>
        {:else}
            <div
                class="relative space-y-6 w-full max-w-md"
                in:fly={{ y: 20, duration: 800 }}
            >
                <div class="space-y-2 text-center">
                    <div
                        class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary/10 mb-4 border border-primary/20"
                    >
                        <Lock class="w-8 h-8 text-primary" />
                    </div>
                    <h1
                        class="text-3xl font-black tracking-tighter text-foreground"
                    >
                        {$t("auth.register.recoveryKey.title")}
                    </h1>
                    <p class="text-muted-foreground text-sm">
                        {$t("auth.register.recoveryKey.description")}
                        <span class="text-primary font-bold"
                            >{$t("auth.register.recoveryKey.keepSecure")}</span
                        >
                    </p>
                </div>

                <Card.Root
                    class="border-primary/20 bg-card/50 backdrop-blur-xl ring-1 ring-primary/10 shadow-2xl"
                >
                    <Card.Content class="pt-8 space-y-6">
                        <div
                            class="bg-zinc-900/50 border border-zinc-800 rounded-2xl p-6 text-center group cursor-pointer hover:border-primary/30 transition-all"
                            onclick={() => {
                                navigator.clipboard.writeText(recoveryKey);
                                toast.success(
                                    $t("auth.register.recoveryKey.successCopy"),
                                );
                            }}
                        >
                            <p
                                class="text-xs font-black uppercase tracking-[0.3em] text-muted-foreground mb-2"
                            >
                                {$t("auth.register.recoveryKey.label")}
                            </p>
                            <div
                                class="text-2xl font-mono font-black tracking-widest text-foreground selection:bg-primary selection:text-white"
                            >
                                {recoveryKey}
                            </div>
                            <p
                                class="text-[10px] text-muted-foreground/60 mt-4 group-hover:text-primary transition-colors"
                            >
                                {$t("auth.register.recoveryKey.copyPrompt")}
                            </p>
                        </div>

                        <div class="space-y-4">
                            <div
                                class="flex items-start gap-3 text-xs text-muted-foreground leading-relaxed italic"
                            >
                                <div
                                    class="mt-1 w-1.5 h-1.5 rounded-full bg-primary flex-shrink-0"
                                ></div>
                                {$t("auth.register.recoveryKey.warning")}
                            </div>
                        </div>

                        <Button
                            class="w-full font-black py-6 text-lg shadow-lg shadow-primary/20"
                            onclick={confirmRecovery}
                        >
                            {$t("auth.register.recoveryKey.confirm")}
                        </Button>
                    </Card.Content>
                </Card.Root>
            </div>
        {/if}
    </div>
</div>
