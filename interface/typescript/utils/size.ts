/** Вспомогательная функция получения размера строки в байтах */
export const getStringSize = (s: string): number => new TextEncoder().encode(s).length;
