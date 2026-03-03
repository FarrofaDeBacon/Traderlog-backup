<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { ChevronLeft, ChevronRight, X } from "lucide-svelte";

    let {
        images = [],
        index = $bindable(null),
        onclose = () => {},
    } = $props<{
        images: string[];
        index: number | null;
        onclose?: () => void;
    }>();

    function close() {
        index = null;
        onclose();
    }

    function next() {
        if (index !== null) {
            index = (index + 1) % images.length;
        }
    }

    function prev() {
        if (index !== null) {
            index = (index - 1 + images.length) % images.length;
        }
    }

    function handleKeydown(event: KeyboardEvent) {
        if (index === null) return;
        if (event.key === "Escape") close();
        if (event.key === "ArrowRight") next();
        if (event.key === "ArrowLeft") prev();
    }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if index !== null}
    <div
        class="fixed inset-0 z-[100] bg-black/95 backdrop-blur-sm flex flex-col items-center justify-center p-4 animate-in fade-in duration-300"
        role="dialog"
        aria-modal="true"
    >
        <div class="absolute top-4 right-4 flex gap-2">
            <Button
                variant="ghost"
                size="icon"
                class="text-white hover:bg-white/10"
                onclick={close}
            >
                <X class="w-6 h-6" />
            </Button>
        </div>

        <div
            class="relative w-full max-w-5xl aspect-video flex items-center justify-center"
        >
            {#if images.length > 1}
                <Button
                    variant="ghost"
                    size="icon"
                    class="absolute left-4 z-10 text-white bg-black/50 hover:bg-black/80 rounded-full h-12 w-12 transition-all"
                    onclick={prev}
                >
                    <ChevronLeft class="w-8 h-8" />
                </Button>
            {/if}

            <img
                src={images[index]}
                alt="Enlarged evidence"
                class="max-w-full max-h-full object-contain rounded-lg shadow-2xl animate-in zoom-in-95 duration-300"
            />

            {#if images.length > 1}
                <Button
                    variant="ghost"
                    size="icon"
                    class="absolute right-4 z-10 text-white bg-black/50 hover:bg-black/80 rounded-full h-12 w-12 transition-all"
                    onclick={next}
                >
                    <ChevronRight class="w-8 h-8" />
                </Button>
            {/if}
        </div>

        <div
            class="mt-6 text-white/50 text-sm font-bold uppercase tracking-[0.2em]"
        >
            {index + 1} / {images.length}
        </div>
    </div>
{/if}
