type DateFormat = "common" | "exact";

/**
 * Функция конвертации даты.
 * 
 * Если дата содержит 1 символ, то к её началу прибавится 0.
 */
const conv = (num: number): string => num.toString().padStart(2, "0");

/** Функция получения текущей даты в определённом формате */
function date(format: DateFormat = "common"): string {
	const date = new Date();
	return format === "common" ? `${conv(date.getHours())}:${conv(date.getMinutes())}:${conv(date.getSeconds())}` : `${conv(date.getDate())}.${conv(date.getMonth() + 1)}.${date.getFullYear().toString()}`;
}

export { date }
