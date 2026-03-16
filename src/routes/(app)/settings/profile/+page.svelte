<script lang="ts">
    import {
        User,
        Mail,
        Phone,
        Lock,
        Palette,
        Globe,
        Save,
        Camera,
        UploadCloud,
        X,
        ShieldCheck,
        LogOut,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import { Separator } from "$lib/components/ui/separator";
    import * as Select from "$lib/components/ui/select";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { settingsStore } from "$lib/stores/settings.svelte";
    import { toast } from "svelte-sonner";
    import * as Avatar from "$lib/components/ui/avatar";
    import { goto } from "$app/navigation";
    import { t, locale } from "svelte-i18n";

    let initialized = $state(false);
    let formData = $state({
        ...settingsStore.userProfile,
        // Garantir que campos críticos não sejam undefined
        name: settingsStore.userProfile?.name || "",
        email: settingsStore.userProfile?.email || "",
        utc_offset: settingsStore.userProfile?.utc_offset ?? -180,
    });

    // Sincroniza uma vez quando os dados reais chegam do backend
    $effect(() => {
        const profile = settingsStore.userProfile;
        if (!initialized && profile && profile.name !== "") {
            formData = { ...profile };
            initialized = true;
        }
    });

    const timezones = [
        {
            id: "-300",
            label: "UTC -5 (Acre)",
            tz: "America/Rio_Branco",
            offset: -300,
        },
        {
            id: "-240",
            label: "UTC -4 (AM/MT/MS)",
            tz: "America/Manaus",
            offset: -240,
        },
        {
            id: "-180",
            label: "UTC -3 (Brasília/SP)",
            tz: "America/Sao_Paulo",
            offset: -180,
        },
        {
            id: "-120",
            label: "UTC -2 (Fernando de Noronha)",
            tz: "America/Noronha",
            offset: -120,
        },
        { id: "0", label: "UTC (Londres/GMT)", tz: "Europe/London", offset: 0 },
        {
            id: "60",
            label: "UTC +1 (Europa Central)",
            tz: "Europe/Paris",
            offset: 60,
        },
    ];

    let currentTZ = $derived(
        timezones.find((t) => t.offset === formData.utc_offset) || {
            label: `${$t("general.custom")} (${formData.utc_offset} min)`,
            tz: formData.timezone,
            offset: formData.utc_offset,
        },
    );

    let securityData = $state({
        current_password: "",
        new_password: "",
        confirm_password: "",
    });

    function handlePasswordChange() {
        if (securityData.new_password.length < 6) {
            toast.error($t("settings.profile.security.error.minChars"));
            return;
        }
        if (securityData.new_password !== securityData.confirm_password) {
            toast.error($t("settings.profile.security.error.mismatch"));
            return;
        }
        toast.success($t("settings.profile.security.success"));
        securityData.current_password = "";
        securityData.new_password = "";
        securityData.confirm_password = "";
    }

    function save() {
        settingsStore.updateUserProfile(formData);
        toast.success($t("settings.profile.success"));
    }

    function handleAvatarUpload(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files?.length) return;
        const file = input.files[0];
        const reader = new FileReader();
        reader.onload = (e) => {
            if (e.target?.result) {
                formData.avatar = e.target.result as string;
            }
        };
        reader.readAsDataURL(file);
        input.value = "";
    }
</script>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <div class="space-y-0.5">
            <h3 class="text-lg font-medium">{$t("settings.profile.title")}</h3>
            <p class="text-sm text-muted-foreground">
                {$t("settings.profile.description")}
            </p>
        </div>
        <Button onclick={save}>
            <Save class="w-4 h-4 mr-2" />
            {$t("settings.profile.save")}
        </Button>
    </div>
    <Separator />

    <div class="grid gap-6 grid-cols-1 lg:grid-cols-3">
        <!-- Left Column: Personal Info & Security -->
        <div class="lg:col-span-2 space-y-6">
            <!-- Personal Info -->
            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class="p-1.5 rounded bg-blue-500/10">
                        <User class="w-4 h-4 text-blue-500" />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight uppercase">
                        {$t("settings.profile.personalInfo.title")}
                    </h4>
                </div>

                <Card.Root>
                    <Card.Content class="space-y-6 pt-6">
                        <!-- Avatar Upload -->
                        <div class="flex items-center gap-4">
                            <div class="relative group">
                                <Avatar.Root
                                    class="w-20 h-20 border-2 border-muted"
                                >
                                    <Avatar.Image
                                        src={formData.avatar}
                                        alt={formData.name}
                                    />
                                    <Avatar.Fallback>
                                        <User class="w-8 h-8 opacity-50" />
                                    </Avatar.Fallback>
                                </Avatar.Root>
                                <label
                                    for="avatar-ops"
                                    class="absolute inset-0 flex items-center justify-center bg-black/50 rounded-full opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer text-white"
                                >
                                    <Camera class="w-6 h-6" />
                                    <input
                                        type="file"
                                        id="avatar-ops"
                                        class="hidden"
                                        accept="image/*"
                                        onchange={handleAvatarUpload}
                                    />
                                </label>
                            </div>
                            <div class="space-y-1">
                                <h4 class="font-medium text-sm">
                                    {$t("settings.profile.personalInfo.avatar")}
                                </h4>
                                <p class="text-xs text-muted-foreground">
                                    {$t(
                                        "settings.profile.personalInfo.avatarHint",
                                    )}
                                </p>
                                {#if formData.avatar}
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        class="h-6 text-xs text-destructive hover:text-destructive px-0"
                                        onclick={() => (formData.avatar = null)}
                                    >
                                        {$t(
                                            "settings.profile.personalInfo.removePhoto",
                                        )}
                                    </Button>
                                {/if}
                            </div>
                        </div>

                        <Separator />

                        <div class="space-y-2">
                            <Label
                                >{$t(
                                    "settings.profile.personalInfo.fullName",
                                )}</Label
                            >
                            <Input
                                bind:value={formData.name}
                                class="font-medium"
                            />
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.personalInfo.cpf",
                                    )}</Label
                                >
                                <Input
                                    bind:value={formData.cpf}
                                    placeholder={$t(
                                        "settings.profile.personalInfo.cpfPlaceholder",
                                    )}
                                />
                            </div>
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.personalInfo.birthDate",
                                    )}</Label
                                >
                                <Input
                                    type="date"
                                    bind:value={formData.birth_date}
                                />
                            </div>
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.personalInfo.email",
                                    )}</Label
                                >
                                <div class="relative">
                                    <Mail
                                        class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
                                    />
                                    <Input
                                        type="email"
                                        bind:value={formData.email}
                                        class="pl-9"
                                    />
                                </div>
                            </div>
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.personalInfo.phone",
                                    )}</Label
                                >
                                <div class="relative">
                                    <Phone
                                        class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
                                    />
                                    <Input
                                        bind:value={formData.phone}
                                        class="pl-9"
                                    />
                                </div>
                            </div>
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>

            <!-- Security / Login -->
            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <div class="p-1.5 rounded bg-yellow-500/10">
                            <Lock class="w-4 h-4 text-yellow-500" />
                        </div>
                        <h4
                            class="text-lg font-semibold tracking-tight uppercase"
                        >
                            {$t("settings.profile.security.title")}
                        </h4>
                    </div>
                    <AlertDialog.Root>
                        <AlertDialog.Trigger>
                            {#snippet child({
                                props,
                            }: {
                                props: Record<string, any>;
                            })}
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    class="text-destructive hover:text-destructive hover:bg-destructive/10"
                                    {...props}
                                >
                                    <LogOut class="w-4 h-4 mr-2" />
                                    {$t("settings.profile.security.logout")}
                                </Button>
                            {/snippet}
                        </AlertDialog.Trigger>
                        <AlertDialog.Content>
                            <AlertDialog.Header>
                                <AlertDialog.Title
                                    >{$t(
                                        "settings.profile.security.logout",
                                    )}</AlertDialog.Title
                                >
                                <AlertDialog.Description>
                                    {$t(
                                        "settings.profile.security.confirmLogout",
                                    )}
                                </AlertDialog.Description>
                            </AlertDialog.Header>
                            <AlertDialog.Footer>
                                <AlertDialog.Cancel
                                    >{$t("general.cancel")}</AlertDialog.Cancel
                                >
                                <AlertDialog.Action
                                    onclick={() => {
                                        toast.success(
                                            $t(
                                                "settings.profile.security.loggingOut",
                                            ),
                                        );
                                        settingsStore.logout();
                                        setTimeout(() => goto("/login"), 1000);
                                    }}
                                    class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                                >
                                    {$t("settings.profile.security.logout")}
                                </AlertDialog.Action>
                            </AlertDialog.Footer>
                        </AlertDialog.Content>
                    </AlertDialog.Root>
                </div>

                <Card.Root>
                    <Card.Content class="space-y-4 pt-6">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div class="space-y-4">
                                <div class="space-y-2">
                                    <Label
                                        >{$t(
                                            "settings.profile.security.currentPassword",
                                        )}</Label
                                    >
                                    <Input
                                        type="password"
                                        bind:value={
                                            securityData.current_password
                                        }
                                        placeholder={$t(
                                            "settings.profile.security.currentPasswordPlaceholder",
                                        )}
                                    />
                                </div>
                            </div>
                            <div class="space-y-4">
                                <div class="space-y-2">
                                    <Label
                                        >{$t(
                                            "settings.profile.security.newPassword",
                                        )}</Label
                                    >
                                    <Input
                                        type="password"
                                        bind:value={securityData.new_password}
                                        placeholder={$t(
                                            "settings.profile.security.newPasswordPlaceholder",
                                        )}
                                    />
                                </div>
                                <div class="space-y-2">
                                    <Label
                                        >{$t(
                                            "settings.profile.security.confirmPassword",
                                        )}</Label
                                    >
                                    <Input
                                        type="password"
                                        bind:value={
                                            securityData.confirm_password
                                        }
                                        placeholder={$t(
                                            "settings.profile.security.confirmPasswordPlaceholder",
                                        )}
                                    />
                                </div>
                                <Button
                                    variant="outline"
                                    class="w-full border-yellow-500/30 text-yellow-500 hover:bg-yellow-500/10"
                                    onclick={handlePasswordChange}
                                >
                                    {$t(
                                        "settings.profile.security.changePassword",
                                    )}
                                </Button>
                            </div>
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>
        </div>

        <!-- Right Column: Settings & License -->
        <div class="space-y-6">
            <!-- System Preferences -->
            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class="p-1.5 rounded bg-indigo-500/10">
                        <Palette class="w-4 h-4 text-indigo-500" />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight uppercase">
                        {$t("settings.profile.preferences.title")}
                    </h4>
                </div>

                <Card.Root>
                    <Card.Content class="space-y-4 pt-6">
                        <div class="space-y-2">
                            <Label
                                >{$t(
                                    "settings.profile.preferences.theme",
                                )}</Label
                            >
                            <Select.Root
                                type="single"
                                bind:value={formData.theme}
                            >
                                <Select.Trigger
                                    >{formData.theme === "dark"
                                        ? $t(
                                              "settings.profile.preferences.themeDark",
                                          )
                                        : $t(
                                              "settings.profile.preferences.themeLight",
                                          )}</Select.Trigger
                                >
                                <Select.Content>
                                    <Select.Item value="dark"
                                        >{$t(
                                            "settings.profile.preferences.themeDark",
                                        )}</Select.Item
                                    >
                                    <Select.Item value="light"
                                        >{$t(
                                            "settings.profile.preferences.themeLight",
                                        )}</Select.Item
                                    >
                                </Select.Content>
                            </Select.Root>
                        </div>

                        <div class="space-y-4">
                            <!-- Language -->
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.preferences.language",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    value={formData.language}
                                    onValueChange={(v) => {
                                        formData.language = v as string;
                                        locale.set(v as string);
                                        localStorage.setItem(
                                            "locale",
                                            v as string,
                                        );
                                        toast.success(
                                            $t("settings.profile.success"),
                                        );
                                    }}
                                >
                                    <Select.Trigger>
                                        {#if formData.language === "pt-BR"}
                                            {$t(
                                                "settings.profile.preferences.langPT",
                                            )}
                                        {:else if formData.language === "en"}
                                            {$t(
                                                "settings.profile.preferences.langEN",
                                            )}
                                        {:else if formData.language === "es-ES"}
                                            {$t(
                                                "settings.profile.preferences.langES",
                                            )}
                                        {:else if formData.language === "fr-FR"}
                                            {$t(
                                                "settings.profile.preferences.langFR",
                                            )}
                                        {/if}
                                    </Select.Trigger>
                                    <Select.Content>
                                        <Select.Item value="pt-BR"
                                            >{$t(
                                                "settings.profile.preferences.langPT",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="en"
                                            >{$t(
                                                "settings.profile.preferences.langEN",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="es-ES"
                                            >{$t(
                                                "settings.profile.preferences.langES",
                                            )}</Select.Item
                                        >
                                        <Select.Item value="fr-FR"
                                            >{$t(
                                                "settings.profile.preferences.langFR",
                                            )}</Select.Item
                                        >
                                    </Select.Content>
                                </Select.Root>
                            </div>

                            <!-- Combined Timezone & Offset Selector -->
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.preferences.utcOffset",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    value={String(formData.utc_offset)}
                                    onValueChange={(v) => {
                                        const selected = timezones.find(
                                            (t) => t.id === v,
                                        );
                                        if (selected) {
                                            formData.utc_offset =
                                                selected.offset;
                                            formData.timezone = selected.tz;
                                        }
                                    }}
                                >
                                    <Select.Trigger>
                                        {currentTZ.label}
                                    </Select.Trigger>
                                    <Select.Content>
                                        {#each timezones as tz}
                                            <Select.Item value={tz.id}
                                                >{tz.label}</Select.Item
                                            >
                                        {/each}
                                    </Select.Content>
                                </Select.Root>
                                <div class="flex items-center gap-2 px-1">
                                    <Globe
                                        class="w-3 h-3 text-muted-foreground"
                                    />
                                    <span
                                        class="text-[10px] text-muted-foreground"
                                    >
                                        {$t(
                                            "settings.profile.preferences.region",
                                        )}: {formData.timezone}
                                    </span>
                                </div>
                            </div>

                            <!-- Main Currency -->
                            <div class="space-y-2">
                                <Label
                                    >{$t(
                                        "settings.profile.preferences.mainCurrency",
                                    )}</Label
                                >
                                <Select.Root
                                    type="single"
                                    bind:value={formData.main_currency}
                                >
                                    <Select.Trigger
                                        >{formData.main_currency}</Select.Trigger
                                    >
                                    <Select.Content>
                                        <Select.Item value="BRL"
                                            >BRL (R$)</Select.Item
                                        >
                                        <Select.Item value="USD"
                                            >USD ($)</Select.Item
                                        >
                                        <Select.Item value="EUR"
                                            >EUR (€)</Select.Item
                                        >
                                    </Select.Content>
                                </Select.Root>
                            </div>
                        </div>

                        <Separator />

                        <div class="flex items-center gap-2">
                            <input
                                type="checkbox"
                                id="convert_all"
                                bind:checked={formData.convert_all_to_main}
                                class="accent-primary w-4 h-4 rounded border-gray-300"
                            />
                            <Label
                                for="convert_all"
                                class="cursor-pointer font-normal"
                                >{$t(
                                    "settings.profile.preferences.convertAll",
                                )}</Label
                            >
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>

            <!-- License -->
            <div class="space-y-4">
                <!-- Rich Header -->
                <div class="flex items-center gap-2">
                    <div class="p-1.5 rounded bg-emerald-500/10">
                        <ShieldCheck class="w-4 h-4 text-emerald-500" />
                    </div>
                    <h4 class="text-lg font-semibold tracking-tight uppercase">
                        {$t("settings.profile.license.title")}
                    </h4>
                </div>

                <Card.Root
                    class="relative overflow-hidden border-emerald-500/20 bg-emerald-500/5"
                >
                    <Card.Content class="pt-6">
                        <div class="flex flex-col gap-4">
                            <div class="space-y-1">
                                <h4
                                    class="font-medium text-emerald-600 dark:text-emerald-400"
                                >
                                    {$t("settings.profile.license.status")}: {settingsStore.licenseStatus ===
                                    "active"
                                        ? $t("settings.profile.license.active")
                                        : settingsStore.licenseStatus ===
                                             "trial"
                                          ? $t("settings.profile.license.trial")
                                          : $t(
                                                "settings.profile.license.expired",
                                            )}
                                </h4>
                                <p class="text-xs text-muted-foreground">
                                    {$t("settings.profile.license.prompt")}
                                </p>
                            </div>
                            <Button
                                variant="outline"
                                class="w-full border-emerald-500/30 text-emerald-500 hover:bg-emerald-500/10"
                                onclick={() => goto("/settings/license")}
                            >
                                {$t("settings.profile.license.viewDetails")}
                            </Button>
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>
        </div>
    </div>
</div>
