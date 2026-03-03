# Development Guide

## Environment Setup
To develop for TraderLog Pro, you need:
- **Rust Toolchain**: `stable-x86_64-pc-windows-msvc` (recommended).
- **Node.js**: v18 or v20.
- **pnpm**: Version 8+.
- **PowerShell 7**: Required for the RTD bridge scripts.

## Core Commands

### Development Server
```bash
# Starts both frontend (Vite) and backend (Tauri)
pnpm tauri dev
```

### Static Build (Frontend only)
```bash
pnpm build
```

### Production Build (Bundle Installer)
```bash
pnpm tauri build
```
*Output will be in `src-tauri/target/release/bundle/msi/`.*

---

## Utility Scripts
The project includes several `scripts/` to automate maintenance:

| Script | Purpose |
|--------|---------|
| `scripts/mock_rtd.ps1` | Injects trades into the bridge for testing without Profit. |
| `scripts/seed_stress_data.py` | Populates the database with 10k trades via the Rust command. |
| `cleanup_locales.cjs` | Syncs keys between `pt-BR.json` and other languages. |
| `scripts/backup_project.ps1` | Creates a compressed backup of the codebase. |

---

## Testing Workflow

### Backend (Rust)
```bash
cd src-tauri
cargo test
```
*Crucial for verifying fiscal logic changes.*

### Frontend (Vitest)
```bash
pnpm test
```
*Verifies stores, parsers, and utility functions.*

### E2E (Playwright)
```bash
pnpm tauri exec playwright test
```
*Runs full browser-automation flows like Onboarding and Login.*

---

## Code Structure Guidelines
1. **Reactivity**: Always use Svelte 5 Runes (`$state`, `$derived`).
2. **Persistence**: Use `invoke` to talk to Rust for anything that needs saving.
3. **Styles**: Use Tailwind CSS v4 classes. Avoid arbitrary values; use the theme tokens.
4. **i18n**: Never hardcode strings. Add them to `src/lib/i18n/locales/pt-BR.json` first.
