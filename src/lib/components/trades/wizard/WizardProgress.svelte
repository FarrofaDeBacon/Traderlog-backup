<script lang="ts">
    let {
        currentStep = 1,
        steps = [],
        onStepClick = (step: number) => {},
    } = $props<{
        currentStep: number;
        steps: { number: number; label: string }[];
        onStepClick?: (step: number) => void;
    }>();
</script>

<div class="flex items-center justify-between mb-8 px-4">
    {#each steps as step, index}
        {@const isActive = currentStep === step.number}
        {@const isCompleted = currentStep > step.number}

        <div
            class="flex items-center {index < steps.length - 1 ? 'flex-1' : ''}"
        >
            <!-- Step Circle -->
            <button
                type="button"
                onclick={() => onStepClick(step.number)}
                class="flex flex-col items-center relative z-10 cursor-pointer hover:scale-105 transition-transform"
            >
                <div
                    class="w-10 h-10 rounded-full flex items-center justify-center font-bold transition-all duration-300 {isActive
                        ? 'bg-primary text-primary-foreground scale-110 shadow-lg shadow-primary/50'
                        : isCompleted
                          ? 'bg-green-500 text-white'
                          : 'bg-zinc-800 text-zinc-400 hover:bg-zinc-700'}"
                >
                    {#if isCompleted}
                        ✓
                    {:else}
                        {step.number}
                    {/if}
                </div>
                <span
                    class="text-[10px] mt-2 font-bold uppercase tracking-wider transition-colors {isActive
                        ? 'text-primary'
                        : 'text-zinc-500'}"
                >
                    {step.label}
                </span>
            </button>

            <!-- Connector Line -->
            {#if index < steps.length - 1}
                <div
                    class="flex-1 h-1 mx-2 rounded-full transition-all duration-300 {isCompleted
                        ? 'bg-green-500'
                        : 'bg-zinc-800'}"
                ></div>
            {/if}
        </div>
    {/each}
</div>
