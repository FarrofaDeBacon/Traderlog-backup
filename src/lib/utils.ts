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
		// Handle null/undefined values early
		const safeAmount = amount ?? 0;
		const safeCurrency = currency || "BRL";
		const isoCurrency = safeCurrency.toUpperCase();

		if (isoCurrency === "USDT") {
			const isNegative = safeAmount < 0;
			const absAmount = Math.abs(safeAmount);

			const formatted = new Intl.NumberFormat(locale, {
				minimumFractionDigits: 2,
				maximumFractionDigits: 2,
			}).format(absAmount);

			return `${isNegative ? "-" : ""}USDT ${formatted}`;
		}

		return new Intl.NumberFormat(locale, {
			style: "currency",
			currency: safeCurrency,
		}).format(safeAmount);
	} catch (e) {
		// Fallback for any other invalid currency codes that might slip through
		const safeAmount = amount ?? 0;
		const safeCurrency = currency || "---";
		return `${safeCurrency} ${new Intl.NumberFormat(locale, { minimumFractionDigits: 2 }).format(safeAmount)}`;
	}
};

export const formatNumber = (amount: number, locale: string = "pt-BR") => {
	return new Intl.NumberFormat(locale, {
		minimumFractionDigits: 2,
		maximumFractionDigits: 2,
	}).format(amount);
};

/**
 * Parses a date string safely, ensuring "YYYY-MM-DD" is treated as local midnight.
 * Direct "YYYY-MM-DD" parsing by new Date() often defaults to UTC midnight, 
 * which shifts the date in local Brazilian timezones.
 */
export const parseSafeDate = (dateStr: string | null | undefined): Date => {
    if (!dateStr) return new Date();

    // If it's strictly YYYY-MM-DD, force it to be local by adding a separator-less time or using parts
    if (/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) {
        const [year, month, day] = dateStr.split("-").map(Number);
        return new Date(year, month - 1, day, 0, 0, 0);
    }

    // Attempt normalized parsing
    const normalized = dateStr.includes(" ") ? dateStr.replace(" ", "T") : dateStr;
    const d = new Date(normalized);

    if (isNaN(d.getTime())) {
        // Fallback for weird formats
        return new Date();
    }

    return d;
};

export const getLocalDatePart = (dateStr: string): string => {
	if (!dateStr) return "";

    const d = parseSafeDate(dateStr);

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
 * Combina uma data YYYY-MM-DD com o horário local atual, adicionando o fuso horário (offset)
 * para evitar bugs de deslocamento temporal quando o backend (SurrealDB) faz o parse da data.
 */
export const formatLocalISO = (dateStr: string): string => {
	if (!dateStr) return "";
	// Se já contiver T, assume que já tem horário e retorna
	if (dateStr.includes("T")) return dateStr;

	const timePart = getCurrentLocalTimePart();

	// Adiciona o timezone offset local
	const now = new Date();
	const offset = -now.getTimezoneOffset(); // em minutos
	const sign = offset >= 0 ? '+' : '-';
	const absOffset = Math.abs(offset);
	const hours = String(Math.floor(absOffset / 60)).padStart(2, "0");
	const minutes = String(absOffset % 60).padStart(2, "0");

	return `${dateStr}T${timePart}${sign}${hours}:${minutes}`;
};

/**
 * Garante que uma string de data (possivelmente de um input datetime-local) 
 * tenha o offset do fuso horário local correto.
 */
export const ensureLocalOffset = (dateStr: string): string => {
    if (!dateStr) return "";
    
    // Se já tiver offset ou for UTC, não mexe
    if (dateStr.includes("+") || (dateStr.includes("-") && dateStr.lastIndexOf("-") > 10) || dateStr.endsWith("Z")) {
        return dateStr;
    }

    // Se for apenas YYYY-MM-DD
    if (/^\d{4}-\d{2}-\d{2}$/.test(dateStr)) {
        return formatLocalISO(dateStr);
    }

    // Se for YYYY-MM-DDTHH:mm ou YYYY-MM-DDTHH:mm:ss
    const now = new Date();
    const offset = -now.getTimezoneOffset();
    const sign = offset >= 0 ? '+' : '-';
    const absOffset = Math.abs(offset);
    const offHours = String(Math.floor(absOffset / 60)).padStart(2, "0");
    const offMinutes = String(absOffset % 60).padStart(2, "0");

    // Garante segundos se faltar
    const normalized = dateStr.split("T").length === 2 && dateStr.split(":").length === 2 
        ? `${dateStr}:00` 
        : dateStr;

    return `${normalized}${sign}${offHours}:${offMinutes}`;
};
