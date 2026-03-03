# 📈 TraderLog Pro (Beta)

> **A evolução do seu diário de trading.** Um ecossistema completo para gestão de performance, controle psicológico e apuração fiscal automatizada para traders da B3.

---

## 🚀 O que é o TraderLog Pro?

O **TraderLog Pro** é uma aplicação desktop (Tauri v2 + Rust) projetada para traders profissionais que buscam excelência operacional. Diferente de diários comuns, ele combina:
- **📊 Dashboard de Performance**: Gráficos avançados, Profit Factor, Payoff e Expectativa Matemática.
- **🔗 Integração RTD**: Detecta execuções no Profit/Excel em tempo real para auto-journaling.
- **⚖️ Motor Fiscal (IRPF)**: Apuração automatizada seguindo regras da Receita Federal (Day Trade vs Swing Trade).
- **🧠 Diário Psicológico**: Monitoramento de mindset e estados emocionais por operação.
- **🔒 Privacidade Total**: Banco de dados SurrealDB local (offline-first). Seus dados não saem da sua máquina.

---

## 🛠️ Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) (Runes), [Tailwind CSS v4](https://tailwindcss.com/), Lucide Icons.
- **Backend**: [Rust](https://www.rust-lang.org/) (Tauri v2), [Tokio](https://tokio.rs/) (Async).
- **Database**: [SurrealDB](https://surrealdb.com/) (Local KV - SurrealKV).
- **Integration**: PowerShell RTD Bridge.

---

## 🏃 Quick Start (Desenvolvedores)

### Pré-requisitos
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (instalador recomendado)

### Rodando o projeto
1. Clone o repositório.
2. Instale as dependências:
   ```bash
   pnpm install
   ```
3. Inicie o ambiente de desenvolvimento (Vite + Tauri):
   ```bash
   pnpm tauri dev
   ```

---

## 📚 Documentações Técnicas

- 🏗️ [Arquitetura de Sistemas](./docs/ARCHITECTURE.md)
- ⚖️ [Lógica do Motor Fiscal IRPF](./docs/FISCAL_ENGINE.md)
- 💻 [Guia de Desenvolvimento e Scripts](./docs/DEVELOPMENT.md)

---

## 🛡️ License

Este software é distribuído sob licença própria (TraderLog License). Veja [LICENSE](./LICENSE) para mais detalhes.
