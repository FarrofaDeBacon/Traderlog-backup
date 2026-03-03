<script lang="ts">
    import { onMount } from "svelte";
    import { fade, scale } from "svelte/transition";
    import { Rocket } from "lucide-svelte";
    import { t } from "svelte-i18n";

    let { onFinish } = $props();
    let show = $state(true);

    onMount(() => {
        const timer = setTimeout(() => {
            show = false;
            setTimeout(onFinish, 500); // Wait for fade out to finish
        }, 2500);
        return () => clearTimeout(timer);
    });
</script>

{#if show}
    <div
        class="fixed inset-0 z-[100] flex items-center justify-center bg-zinc-950 overflow-hidden"
        out:fade={{ duration: 500 }}
    >
        <!-- Ambient background glow -->
        <div class="absolute inset-0 overflow-hidden pointer-events-none">
            <div
                class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-primary/20 rounded-full blur-[120px] animate-pulse"
            ></div>
        </div>

        <div
            class="relative flex flex-col items-center gap-6"
            in:scale={{ duration: 1000, start: 0.8 }}
        >
            <!-- Animated Logo Container -->
            <div class="relative w-24 h-24">
                <!-- Orbiting Rings -->
                <div
                    class="absolute inset-0 border-2 border-primary/30 rounded-2xl animate-[spin_8s_linear_infinite]"
                ></div>
                <div
                    class="absolute inset-2 border-2 border-primary/50 rounded-xl animate-[spin_12s_linear_infinite_reverse]"
                ></div>

                <!-- Center Icon -->
                <div
                    class="absolute inset-0 flex items-center justify-center bg-primary/10 rounded-2xl border border-primary/20 backdrop-blur-sm shadow-[0_0_40px_-10px_rgba(59,130,246,0.5)]"
                >
                    <Rocket class="w-12 h-12 text-primary" />
                </div>
            </div>

            <!-- Text Branding -->
            <div class="text-center space-y-1">
                <h1
                    class="text-4xl font-black tracking-tighter text-white flex items-center gap-2"
                >
                    <span
                        >Trader<span class="text-primary italic">Log</span> Pro</span
                    >
                    <span
                        class="text-[10px] font-black uppercase tracking-wider text-primary px-1.5 py-0.5 rounded border border-primary/30 bg-primary/10"
                        >Beta</span
                    >
                </h1>
                <div class="flex items-center justify-center gap-2">
                    <div class="h-[1px] w-8 bg-zinc-800"></div>
                    <p
                        class="text-[10px] font-black uppercase tracking-[0.3em] text-zinc-500"
                    >
                        {$t("setup.splash.tagline")}
                    </p>
                    <div class="h-[1px] w-8 bg-zinc-800"></div>
                </div>
            </div>

            <!-- Loading Indicator -->
            <div
                class="absolute -bottom-24 left-1/2 -translate-x-1/2 w-48 h-1 bg-zinc-900 rounded-full overflow-hidden"
            >
                <div
                    class="h-full bg-primary animate-[loading_2.5s_ease-in-out_infinite]"
                ></div>
            </div>
        </div>
    </div>
{/if}

<style>
    @keyframes loading {
        0% {
            transform: translateX(-100%);
        }
        50% {
            transform: translateX(0);
        }
        100% {
            transform: translateX(100%);
        }
    }
</style>
