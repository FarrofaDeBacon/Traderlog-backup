import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

export const formatCurrency = (amount: number, currency: string = "BRL", locale: string = "pt-BR") => {
	try {
		const isoCurrency = currency.toUpperCase();

		if (isoCurrency === "USDT") {
			const isNegative = amount < 0;
			const absAmount = Math.abs(amount);

			const formatted = new Intl.NumberFormat(locale, {
				minimumFractionDigits: 2,
				maximumFractionDigits: 2,
			}).format(absAmount);

			return `${isNegative ? "-" : ""}USDT ${formatted}`;
		}

		return new Intl.NumberFormat(locale, {
			style: "currency",
			currency: currency,
		}).format(amount);
	} catch (e) {
		// Fallback for any other invalid currency codes that might slip through
		// console.warn(`[formatCurrency] Invalid currency code: ${currency}. Falling back.`, e);
		// If currency is a word like "Múltiplas", just append it.
		return `${currency} ${new Intl.NumberFormat(locale, { minimumFractionDigits: 2 }).format(amount)}`;
	}
};

export const formatNumber = (amount: number, locale: string = "pt-BR") => {
	return new Intl.NumberFormat(locale, {
		minimumFractionDigits: 2,
		maximumFractionDigits: 2,
	}).format(amount);
};

export const getLocalDatePart = (dateStr: string): string => {
	if (!dateStr) return "";

	// Se já for apenas uma data YYYY-MM-DD (10 caracteres), retorna como está.
	// Isso evita que o construtor Date trate como UTC e mude o dia.
	if (/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) {
		return dateStr;
	}

	// Tenta normalizar espaços para T para melhor aceitação do construtor Date
	const normalized = dateStr.includes(" ") ? dateStr.replace(" ", "T") : dateStr;
	const d = new Date(normalized);

	if (isNaN(d.getTime())) {
		// Fallback: se não for uma data válida, tenta o split básico
		return dateStr.split(/[ T]/)[0];
	}

	// Retorna a data no fuso horário local
	const year = d.getFullYear();
	const month = String(d.getMonth() + 1).padStart(2, "0");
	const day = String(d.getDate()).padStart(2, "0");

	return `${year}-${month}-${day}`;
};

/**
 * Retorna o horário local atual formatado como HH:mm:ss.SSS
 */
export const getCurrentLocalTimePart = (): string => {
	const now = new Date();
	const hours = String(now.getHours()).padStart(2, "0");
	const minutes = String(now.getMinutes()).padStart(2, "0");
	const seconds = String(now.getSeconds()).padStart(2, "0");
	const ms = String(now.getMilliseconds()).padStart(3, "0");
	return `${hours}:${minutes}:${seconds}.${ms}`;
};

/**
 * Combina uma data YYYY-MM-DD com o horário local atual
 */
export const formatLocalISO = (dateStr: string): string => {
	if (!dateStr) return "";
	// Se já contiver T, assume que já tem horário e retorna
	if (dateStr.includes("T")) return dateStr;

	const timePart = getCurrentLocalTimePart();
	return `${dateStr}T${timePart}`;
};
