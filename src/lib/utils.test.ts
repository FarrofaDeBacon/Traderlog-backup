
import { describe, it, expect } from 'vitest';
import { formatCurrency, formatNumber, getLocalDatePart } from '$lib/utils';

describe('Utils', () => {
    describe('formatCurrency', () => {
        it('should format BRL correctly by default', () => {
            expect(formatCurrency(1234.56)).toContain('R$');
            expect(formatCurrency(1234.56).replace(/\u00A0/g, ' ')).toContain('1.234,56');
        });

        it('should format USDT specially', () => {
            expect(formatCurrency(100, 'USDT')).toBe('USDT 100,00');
            expect(formatCurrency(-50, 'USDT')).toBe('-USDT 50,00');
        });

        it('should handle zero gracefully', () => {
            expect(formatCurrency(0)).toContain('0,00');
        });
    });

    describe('formatNumber', () => {
        it('should format numbers with 2 decimal places', () => {
            expect(formatNumber(1234.5678)).toBe('1.234,57'); // Rounding
            expect(formatNumber(100)).toBe('100,00');
        });
    });

    describe('getLocalDatePart', () => {
        it('should return YYYY-MM-DD for valid inputs', () => {
            expect(getLocalDatePart('2023-12-25')).toBe('2023-12-25');
            expect(getLocalDatePart('2023-12-25T10:00:00.000Z')).toBe('2023-12-25');
        });

        it('should handle dates with time correctly based on local time', () => {
            // Create a date that might shift if UTC was used blindly
            const date = new Date(2023, 0, 1, 12, 0, 0); // Jan 1st 2023 12:00 local
            const iso = date.toISOString();
            const result = getLocalDatePart(iso);
            expect(result).toBe('2023-01-01');
        });
    });
});
