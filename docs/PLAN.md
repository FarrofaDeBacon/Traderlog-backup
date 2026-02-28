# PLAN: Full Application Translation Sweep (ORCHESTRATED)

Systematic audit and internationalization of all remaining hardcoded strings in the TraderLogPro application.

## Phase 1: Planning & Deep Discovery (ORCHESTRATION - SEQUENTIAL)

### 1. Global Codebase Sweep (`explorer-agent`)
- Search ALL `.svelte` and `.ts` files in `src/` for hardcoded strings.
- Detect Portuguese strings (with accents) and English literals.
- Identify "Raw Keys" being displayed incorrectly in the UI.

### 2. Full Task Generation (`project-planner`)
- Categorize discovered strings by feature area.
- Populate `task.md` with implementation steps.

---

## ⏸️ User Checkpoint
**Wait for discovery report and plan approval.**

---

## Phase 2: Implementation (ORCHESTRATION - PARALLEL)

### 1. Key Extraction & Locale Sync (`backend-specialist`)
- Generate translation keys and update all 4 locale files (`pt-BR`, `en-US`, `es-ES`, `fr-FR`).

### 2. Component Refactoring (`frontend-specialist`)
- Replace strings with `$t` keys in all identified components.
- Fix UI logic where keys are showing instead of values.

### 3. Verification & UI Polish (`test-engineer`)
- Run consistency scripts and perform visual verification.

---

## Deliverables
- 100% i18n coverage across the entire application.
- Synchronized dictionaries for all 4 supported languages.
