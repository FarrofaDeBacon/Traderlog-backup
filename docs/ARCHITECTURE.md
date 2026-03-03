# System Architecture

## Overview
TraderLog Pro is a multi-tier desktop application designed for high performance and strict data privacy. It leverages the **Tauri v2** framework to bridge a high-performance **Rust backend** with a reactive **Svelte 5 frontend**.

## Architecture Diagram

```mermaid
graph TD
    subgraph Windows Environment
        Excel["Profit / Excel (RTD Source)"]
        Bridge["PowerShell RTD Bridge"]
        Excel -->|COM Interface| Bridge
    end

    subgraph Tauri Application (Backend)
        RustCore["Rust Engine (Tokio)"]
        Monitor["RTD Monitor Loop"]
        SurrealDB[(SurrealDB - Local KV)]
        
        Bridge -->|CSV Stream| Monitor
        Monitor --> RustCore
        RustCore -->|Persistence| SurrealDB
    end

    subgraph User Interface (Frontend)
        Svelte["Svelte 5 (Runes)"]
        Stores["Reactivity Stores"]
        
        RustCore -->|Tauri Events| Stores
        Stores --> Svelte
        Svelte -->|App Commands| RustCore
    end
```

## Key Components

### 1. Data Ingestion (RTD Bridge)
Because trading platforms like Nelogica Profit use COM/DDE (Windows-specific), we use a **PowerShell bridge** (`scripts/rtd_bridge.ps1`) as an intermediary. It reads the real-time data from a helper Excel sheet and writes buffered snapshots to a local CSV in `$TEMP`.

### 2. Backend Engine (Rust)
The Rust layer is the source of truth for:
- **RTD Monitoring**: A background thread detects changes in the bridge CSV and emits events to the frontend.
- **Fiscal Calculation**: A specialized module (`irpf.rs`) implements the Brazilian tax logic.
- **Data Persistence**: Uses SurrealDB in local mode, ensuring no external API calls are needed for core functionality.

### 3. Reactive UI (Svelte 5)
The frontend uses **Svelte 5 Runes** (`$state`, `$derived`, `$effect`) to handle massive amounts of real-time data without UI freezing.
- **Stores**: Functional stores manage the state of trades, equity curves, and system settings.
- **Components**: Stylized with Tailwind CSS v4 for a premium, high-density dashboard experience.

## Persistence Strategy
- **SurrealDB**: Used as a local Key-Value store with SQL-like query capabilities.
- **Namespaces**: `traderlog`
- **Databases**: `main`
- **Schemas**: Schema-full for critical entities (Trades, Accounts) to ensure data integrity.

## Security
- All sensitive data (email hashes, hardware IDs) is processed locally.
- Hardware-bound licensing prevents unauthorized distribution while maintaining privacy.
