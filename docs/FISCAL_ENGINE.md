# Fiscal Engine (IRPF) Specification

## Introduction
The TraderLog Pro Fiscal Engine is a robust Rust-based module designed to calculate tax obligations for Brazilian traders (B3) according to the latest Receita Federal regulations.

## Core Tax Logic

### 1. Modality Classification
Operations are strictly separated into two buckets:
- **Day Trade**: Buy and sell executed on the same day. Taxed at **20%**.
- **Swing Trade**: Positions held for more than one day. Taxed at **15%**.

### 2. Exemption Rules (Swing Trade)
- **<= R$ 20,000.00**: Monthly sales (liquidations) of stocks/ETFs/Options up to this threshold are **exempt** from tax in Swing Trade.
- **> R$ 20,000.00**: The entire profit is taxable.
- *Note*: Day Trade transactions are **never** exempt, regardless of the volume.

### 3. Loss Carryover (Compensação)
- Losses in Day Trade can only offset gains in Day Trade.
- Losses in Swing Trade can only offset gains in Swing Trade.
- Losses can be carried over indefinitely to future months.
- TraderLog Pro automatically manages the "Negative Balance" (`prejuízo acumulado`) across months.

### 4. IRRF (Imposto Retido na Fonte)
- **Day Trade**: 1% withheld on profit.
- **Swing Trade**: 0.005% withheld on sales.
- The engine supports manual input or automatic estimation of IRRF, which is then credited against the total tax due (Imposto a Pagar).

## Calculation Flow

1. **Transaction Ingestion**: Trades are grouped by Asset Type and Modality.
2. **Gross Profit Calculation**: Calculated based on Average Cost (Preço Médio) or specific trade matching.
3. **Loss Compensation**: Checks if there's any negative balance from previous months to deduct.
4. **Exemption Check**: Applies the 20k rule for Swing Trade.
5. **Tax Base (Base de Cálculo)**: Final value after losses and exemptions.
6. **Rate Application**: Multiplies by 15% or 20%.
7. **Credit Deduction**: Subtracts IRRF already paid at the source.
8. **DARF Generation**: Final value formatted for payment.

## Edge Cases Verified
- **Wash Sales**: Proper cost adjustment.
- **Min Tax (R$ 10.00)**: If the total tax due is less than R$ 10.00, it is carried over to the next month until the threshold is met.
- **Asset Conversions**: Handling of stock splits and reverse splits via manual adjustment records.

## Automated Verification
All fiscal logic is backed by **Rust unit tests** (`logic_tests.rs`), ensuring that even the most complex carryover scenarios remain accurate after code changes.
