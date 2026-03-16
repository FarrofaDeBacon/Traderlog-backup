<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { readTextFile } from "@tauri-apps/plugin-fs";
    import { resolveResource } from "@tauri-apps/api/path";
    import { Input } from "$lib/components/ui/input";
    import { Search, Book, ChevronRight, X } from "lucide-svelte";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { t } from "svelte-i18n";

    interface HelpSection {
        id: string;
        title: string;
        level: number;
        content: string;
        fullText: string;
    }

    let rawMarkdown = $state("");
    let sections: HelpSection[] = $state([]);
    let searchQuery = $state("");
    let filteredSections = $derived(
        searchQuery.trim() === ""
            ? sections
            : sections.filter(s => 
                s.title.toLowerCase().includes(searchQuery.toLowerCase()) || 
                s.fullText.toLowerCase().includes(searchQuery.toLowerCase())
            )
    );

    let activeSectionId = $state("");
    let selectedImage = $state<string | null>(null);

    onMount(() => {
        // Event listener for images in markdown
        const handleOpenImage = (e: any) => {
            selectedImage = e.detail;
        };
        window.addEventListener('open-help-image', handleOpenImage);

        // Async loading logic
        async function loadManual() {
            try {
                // Fetch from static folder which is served at /docs/
                const response = await fetch("/docs/USER_MANUAL.md");
                if (!response.ok) throw new Error("Manual not found in static/docs/");
                
                const content = await response.text();
                rawMarkdown = content;
                parseMarkdown(content);

                // Sync with URL query parameter
                const q = $page.url.searchParams.get("q");
                if (q) {
                    searchQuery = q;
                }
            } catch (error) {
                console.error("Failed to load user manual via fetch, trying FS:", error);
                try {
                    // Fallback to Tauri FS if fetch fails
                    const content = await readTextFile("docs/USER_MANUAL.md");
                    rawMarkdown = content;
                    parseMarkdown(content);
                } catch (fsError) {
                    console.error("FS fallback also failed:", fsError);
                }
            }
        }

        loadManual();

        return () => {
            window.removeEventListener('open-help-image', handleOpenImage);
        };
    });

    function parseMarkdown(md: string) {
        const lines = md.split("\n");
        const parsedSections: HelpSection[] = [];
        let currentSection: HelpSection | null = null;

        lines.forEach((line) => {
            const headingMatch = line.match(/^(#{1,4})\s+(.*)/);
            if (headingMatch) {
                const level = headingMatch[1].length;
                const title = headingMatch[2];
                const id = title.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
                
                currentSection = {
                    id,
                    title,
                    level,
                    content: "",
                    fullText: title
                };
                parsedSections.push(currentSection);
            } else if (currentSection) {
                currentSection.content += line + "\n";
                currentSection.fullText += " " + line;
            }
        });

        sections = parsedSections;
        if (sections.length > 0) activeSectionId = sections[0].id;
    }

    // Simple markdown highlighting for specific tags
    function formatContent(text: string) {
        return text
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.*?)\*/g, '<em>$1</em>')
            .replace(/!\[(.*?)\]\((.*?)\)/g, (match, alt, src) => {
                // If it's a relative path starting with ./ or assets/
                let cleanSrc = src;
                if (src.startsWith("./")) {
                    cleanSrc = "/docs/" + src.substring(2);
                } else if (src.startsWith("assets/")) {
                    cleanSrc = "/docs/" + src;
                }
                
                return `<div class="my-10 flex flex-col items-center gap-4 group">
                    <div class="relative overflow-hidden rounded-xl border border-border shadow-sm bg-muted/20">
                        <img 
                            src="${cleanSrc}" 
                            alt="${alt}" 
                            class="max-w-full max-h-[500px] object-contain cursor-zoom-in transition-opacity hover:opacity-90" 
                            onclick="window.dispatchEvent(new CustomEvent('open-help-image', { detail: '${cleanSrc}' }))" 
                        />
                    </div>
                    <span class="text-xs font-bold uppercase tracking-widest text-muted-foreground/50 px-8">--- ${alt} ---</span>
                </div>`;
            })
            .replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2" class="text-primary hover:underline font-bold">$1</a>')
            .replace(/`(.*?)`/g, '<code class="bg-muted px-1.5 py-0.5 rounded text-sm font-mono">$1</code>');
    }

    function scrollToSection(id: string) {
        activeSectionId = id;
        const element = document.getElementById(id);
        if (element) {
            // Calculate header offset if needed, but scroll-mt-20 handles it
            element.scrollIntoView({ behavior: "smooth", block: "start" });
        }
    }
</script>

<div class="flex flex-col gap-8 pb-10">
    <div class="flex flex-col gap-2">
        <h1 class="text-4xl font-extrabold tracking-tight">{$t("help.title")}</h1>
        <p class="text-xl text-muted-foreground">{$t("help.subtitle")}</p>
    </div>

    <div class="relative w-full max-w-2xl sticky top-0 z-10 bg-background/80 backdrop-blur-md py-4 -mx-4 px-4">
        <Search class="absolute left-7 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
        <Input 
            type="search" 
            placeholder={$t("help.searchHint")} 
            class="pl-12 h-14 text-lg rounded-full shadow-sm border-primary/20 focus-visible:ring-primary"
            bind:value={searchQuery}
        />
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-12 items-start">
        <!-- Sidebar Navigation (Sticky) -->
        <aside class="hidden lg:flex flex-col gap-6 sticky top-24 max-h-[calc(100vh-120px)] overflow-hidden">
            <div class="flex items-center gap-2 text-sm font-bold uppercase tracking-widest text-primary/70">
                <Book class="h-4 w-4" />
                {$t("help.topics")}
            </div>
            <ScrollArea class="flex-1 pr-4">
                <nav class="flex flex-col gap-1.5">
                    {#each sections as section}
                        <button 
                            class="text-left px-3 py-2.5 rounded-lg text-sm transition-all hover:bg-accent group {activeSectionId === section.id ? 'bg-primary/10 text-primary font-bold shadow-sm' : 'text-muted-foreground/80'}"
                            style="margin-left: {(section.level - 1) * 16}px"
                            onclick={() => scrollToSection(section.id)}
                        >
                            <span class="line-clamp-2">{section.title}</span>
                        </button>
                    {/each}
                </nav>
            </ScrollArea>
        </aside>

        <!-- Content Area -->
        <div class="flex flex-col gap-16 max-w-4xl">
            {#if filteredSections.length === 0}
                <div class="flex flex-col items-center justify-center py-32 text-center bg-muted/30 rounded-3xl border border-dashed">
                    <div class="bg-muted p-6 rounded-full mb-6">
                        <Search class="h-10 w-10 text-muted-foreground" />
                    </div>
                    <h3 class="text-2xl font-bold">{$t("help.emptyState")}</h3>
                    <p class="text-muted-foreground text-lg">{$t("help.emptyStateDescription")}</p>
                </div>
            {:else}
                {#each filteredSections as section}
                    <section id={section.id} class="scroll-mt-32 group">
                        {#if section.level === 1}
                            <h2 class="text-4xl font-black mb-10 border-b-4 border-primary/20 pb-6 flex items-center gap-4">
                                <span class="bg-primary text-primary-foreground h-10 w-1 px-0 rounded-full"></span>
                                {section.title}
                            </h2>
                        {:else if section.level === 2}
                            <h3 class="text-2xl font-bold mb-6 mt-12 text-foreground/90 group-hover:text-primary transition-colors flex items-center gap-2">
                                <ChevronRight class="h-6 w-6 text-primary/50" />
                                {section.title}
                            </h3>
                        {:else if section.level === 3}
                            <h4 class="text-xl font-bold mb-4 mt-8 text-foreground/80">{section.title}</h4>
                        {:else}
                            <h5 class="text-lg font-bold mb-3 mt-6 text-foreground/70">{section.title}</h5>
                        {/if}
                        
                        <div class="text-muted-foreground text-lg leading-relaxed whitespace-pre-wrap font-medium">
                            {@html formatContent(section.content)}
                        </div>
                    </section>
                {/each}
            {/if}
        </div>
    </div>
</div>

{#if selectedImage}
    <!-- Image Modal Overlay -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
        class="fixed inset-0 z-[100] flex items-center justify-center bg-background/95 backdrop-blur-md p-4 md:p-12 animate-in fade-in zoom-in duration-200"
        onclick={() => selectedImage = null}
    >
        <button 
            class="absolute top-6 right-6 p-4 rounded-full bg-background/50 hover:bg-background border shadow-xl transition-all hover:scale-110 active:scale-95 z-[110]"
            onclick={() => selectedImage = null}
        >
            <X class="h-8 w-8 text-foreground" />
        </button>
        
        <img 
            src={selectedImage} 
            alt="Full size manual screenshot" 
            class="max-w-full max-h-full object-contain shadow-2xl rounded-lg animate-in slide-in-from-bottom-4 duration-300"
        />
    </div>
{/if}

<style>
    /* Custom styles for the markdown content */
    :global(section strong) {
        color: hsl(var(--foreground));
        font-weight: 700;
    }
    
    :global(section code) {
        font-family: 'JetBrains Mono', monospace;
        font-size: 0.85em;
        padding: 0.2rem 0.4rem;
        background-color: hsl(var(--muted));
        border-radius: 0.4rem;
        border: 1px solid hsl(var(--border));
    }
    
    :global(section img) {
        cursor: pointer;
    }

    :global(section a) {
        font-weight: 600;
    }
</style>
