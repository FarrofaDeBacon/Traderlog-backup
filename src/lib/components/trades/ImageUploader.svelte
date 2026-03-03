<script lang="ts">
    import { t } from "svelte-i18n";
    import { Button } from "$lib/components/ui/button";
    import { Camera, Trash2, Eye, Plus, FileImage } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import ImageCarousel from "$lib/components/ui/ImageCarousel.svelte";

    let { images = $bindable([]) } = $props<{
        images: string[];
    }>();

    let isDragging = $state(false);

    async function handleFileChange(event: Event) {
        const input = event.target as HTMLInputElement;
        if (!input.files?.length) return;

        await processFiles(input.files);
    }

    async function processFiles(files: FileList) {
        const newImages: string[] = [];

        for (let i = 0; i < files.length; i++) {
            const file = files[i];
            if (!file.type.startsWith("image/")) {
                toast.error(
                    `${file.name} ${$t("trades.wizard.sections.visual_evidence.invalid_image")}`,
                );
                continue;
            }

            try {
                const reader = new FileReader();
                const imageData = await new Promise<string>((resolve) => {
                    reader.onload = () => resolve(reader.result as string);
                    reader.readAsDataURL(file);
                });
                newImages.push(imageData);
            } catch (e) {
                toast.error(
                    `${$t("trades.wizard.sections.visual_evidence.process_error")} ${file.name}`,
                );
            }
        }

        images = [...images, ...newImages];
        if (newImages.length > 0)
            toast.success(
                `${newImages.length} ${$t("trades.wizard.sections.visual_evidence.images_added")}`,
            );
    }

    function removeImage(index: number) {
        images = images.filter((_: string, i: number) => i !== index);
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        isDragging = false;
        if (event.dataTransfer?.files) {
            processFiles(event.dataTransfer.files);
        }
    }

    // Carousel State
    let selectedImageIndex = $state<number | null>(null);
</script>

<div class="space-y-6">
    <div
        role="presentation"
        class="relative min-h-[160px] flex flex-col items-center justify-center border-2 border-dashed rounded-xl transition-all duration-300
        {isDragging
            ? 'border-primary bg-primary/5 scale-[1.01]'
            : 'border-muted bg-muted/5 hover:bg-muted/10'} pointer-events-auto"
        ondragover={(e) => {
            e.preventDefault();
            isDragging = true;
        }}
        ondragleave={() => (isDragging = false)}
        ondrop={handleDrop}
    >
        <input
            type="file"
            multiple
            accept="image/*"
            class="absolute inset-0 opacity-0 cursor-pointer"
            onchange={handleFileChange}
        />

        <div class="flex flex-col items-center gap-3 text-center p-6">
            <div
                class="w-12 h-12 rounded-full bg-muted/20 flex items-center justify-center"
            >
                <Camera class="w-6 h-6 text-muted-foreground" />
            </div>
            <div class="space-y-1">
                <p class="text-sm font-bold uppercase tracking-widest">
                    {$t("trades.wizard.sections.visual_evidence.drag_drop")}
                </p>
                <p class="text-xs text-muted-foreground">
                    {$t("trades.wizard.sections.visual_evidence.description")}
                </p>
            </div>
            <Button
                variant="secondary"
                size="sm"
                class="mt-2 h-8 font-bold pointer-events-none"
            >
                <Plus class="w-3 h-3 mr-2" />
                {$t("trades.wizard.sections.visual_evidence.select_files")}
            </Button>
        </div>
    </div>

    {#if images.length > 0}
        <div
            class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4 animate-in fade-in duration-500"
        >
            {#each images as img, i}
                <div
                    class="relative aspect-video rounded-lg overflow-hidden border border-muted bg-card group shadow-lg"
                >
                    <img
                        src={img}
                        alt="Trade evidence"
                        class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                    />

                    <div
                        class="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-3"
                    >
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-8 w-8 text-white hover:bg-white/20"
                            onclick={() => (selectedImageIndex = i)}
                        >
                            <Eye class="w-4 h-4" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="h-8 w-8 text-red-500 hover:bg-red-500/20"
                            onclick={() => removeImage(i)}
                        >
                            <Trash2 class="w-4 h-4" />
                        </Button>
                    </div>

                    <div
                        class="absolute bottom-2 left-2 px-2 py-0.5 bg-black/50 backdrop-blur-md rounded text-[9px] font-bold text-white/70 uppercase"
                    >
                        {$t(
                            "trades.wizard.sections.visual_evidence.attachment",
                        )} #{i + 1}
                    </div>
                </div>
            {/each}
        </div>
    {/if}
</div>

<ImageCarousel {images} bind:index={selectedImageIndex} />
