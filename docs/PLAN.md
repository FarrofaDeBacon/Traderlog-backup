# Plano de Orquestração: Verificação e Validação da Apuração IRPF

## 1. Escopo do Problema
O usuário solicitou uma verificação da funcionalidade de **Importação/Apuração do IRPF**. 
Com base na análise do código, a "importação" não se trata do upload de um arquivo CSV ou PDF, mas sim da **importação de dados internos da tabela de `trade`** (que já foram previamente importados de relatórios RTD ou inseridos manualmente na aba "Trades"). 

A tela `fiscal/irpf/+page.svelte` possui um botão **"Nova Apuração"** que abre um modal, solicita Mês/Ano, e chama a função Rust `calculate_monthly_tax`. Esta função:
1. Puxa os trades fechados daquele mês.
2. Agrupa por perfil tributário (Swing Trade, Day Trade, FIIs).
3. Calcula lucros, subtrai taxas, abate prejuízos acumulados e calcula o imposto devido gerando as `TaxAppraisal` (Apurações).

## 2. O que precisa ser Verificado/Corrigido
A lógica de banco de dados (que engolia erros silenciosos e falhava nos saves) foi recentemente reescrita para usar a Rust SDK (`db.upsert()`). O objetivo desta orquestração é garantir que O Motor de Impostos (IRPF) funcione de ponta a ponta com a nova lógica.

### Tarefas
- [ ] **Seeding Inicial (`database-architect`)**: Criar as Regras Fiscais Brasileiras (Ex: Day Trade 20%, Swing Trade 15% com isenção de 20k, FIIs 20% sem isenção). Criar os "Tax Profiles" e associá-los aos "Asset Types" (Ações, Futuros, FIIs, etc) diretamente no seed de instalação.
- [ ] **Teste de Validação Backend (`test-engineer`)**: Simular ou analisar o código para verificar se as queries do motor dinâmico de IRPF que usavam formatação SQL (como `SELECT * FROM trade WHERE month = X`) conseguem ler os IDs corretos e cruzar relações de Perfil de Taxa (Tax Profile, Tax Rule).
- [ ] **Refatoração Preemptiva de Save (`backend-specialist`)**: Todos os salvamentos da tela, como `save_appraisal` e `generate_darf`, além de `mark_darf_paid`, já usam a função `upsert_record` nova? É preciso auditar o arquivo `irpf.rs` no backend para garantir que nenhum `UPSERT text` problemático sobreviveu lá.
- [ ] **Integração UI (`frontend-specialist`)**: Garantir que, ao gerar a apuração e salvá-la, a mudança apareça reativamente no Painel Principal sem sumir após o refresh da página.

## 3. Seleção de Agentes (Fase 2)
Caso aprovado, orquestraremos os seguintes agentes:
1. `backend-specialist` -> Auditará e refatorará qualquer query residual no `irpf.rs` para o padrão seguro SDK.
2. `test-engineer` -> Fará o rastreio visual nos logs ou script para garantir que os cálculos batem com o modelo do banco.
3. `frontend-specialist` -> Garantir a reatividade da tela sem a necessidade do usuário pressionar F5 (ex: os KPIs cards "Total Devido", "Total Pago").

---
**Aguardando aprovação do usuário para avançar para a FASE 2.**
