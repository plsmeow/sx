import { invoke } from "@tauri-apps/api/core";

import { getStringSize } from "./size";
import { date } from "./date";
import { CLIENT_VERSION } from "../version";

/** Тип времени */
export enum TimeType {
	Day,
	Hours,
}

/** Структура данных результата функции `getCacheOrNew` */
interface CacheOrNewResult<T> {
	data: T | null;
	size: string;
}

/** Структура данных результата функции загрузки кэша */
export interface LcfResult {
	data: any;
	expired: boolean;
	incompatible: boolean;
}

/** Структура данных результата функции загрузки актуальной информации */
export interface LnfResult {
	data: any;
	error: string | null;
}

/** Функция проверки даты, истекла та или нет */
function isDataExpired(
	saveDateString: string,
	maxStorageValue: number,
	timeType: TimeType
): boolean {
	const dateRegex = /^(\d{1,2})\.(\d{1,2})\.(\d{4})-(\d{1,2}):(\d{1,2}):(\d{1,2})$/;
	const match = saveDateString.match(dateRegex);
	if (!match) return false;

	const [, day, month, year, hours, minutes, seconds] = match.map(Number);
	const saveDate = new Date(year, month - 1, day, hours, minutes, seconds);
	if (isNaN(saveDate.getTime())) return false;

	const nowDate = new Date();
	const expirationDate = new Date(saveDate);

	if (timeType === TimeType.Day) {
		expirationDate.setDate(expirationDate.getDate() + maxStorageValue);
	} else if (timeType === TimeType.Hours) {
		expirationDate.setHours(expirationDate.getHours() + maxStorageValue);
	} else {
		return false;
	}

	return nowDate > expirationDate;
}

/** Функция сохранения кэша */
export async function saveCache(cache: string, filename: string): Promise<CommandResult<null>> {
	const encoder = new TextEncoder();
	const uint8 = encoder.encode(cache);

	return await invoke("save_cache", {
		cache: Array.from(uint8),
		filename: filename,
		clientVersion: CLIENT_VERSION,
		date: `${date("exact")}-${date("common")}`,
	});
}

/** Функция загрузки кэша */
export async function loadCache(name: string, filename: string, maxStorageValue: number, timeType: TimeType): Promise<CommandResult<string>> {
	const result1 = await invoke<CommandResult<string>>("extract_cache_meta", { name: name });

	if (!result1.data) {
		return {
			data: null,
			error: result1.error,
		};
	}

	if (result1.data.split(" ")[0] !== CLIENT_VERSION) {
		return {
			data: null,
			error: "incompatible",
		};
	}

	const result2 = await invoke<CommandResult<number[]>>("load_cache", { filename: filename });

	if (!result2.data) {
		return {
			data: null,
			error: result2.error,
		};
	}

	const uint8arr = new Uint8Array(result2.data);
	const decoder = new TextDecoder("utf-8");
	const cacheStr = decoder.decode(uint8arr);

	return {
		data: cacheStr,
		error: isDataExpired(result1.data, maxStorageValue, timeType) ? "expired" : null,
	};
}

/** Вспомогательная функция конвертации данных в строку */
const stringify = (data: any): string => typeof data === "object" ? JSON.stringify(data) : typeof data === "number" ? data.toString() : data as string;

/** Функция загрузки кэшированной или новой информации */
export async function loadCacheOrNew<T>(lcf: Function, lnf: Function, scf: Function, onerr: Function): Promise<CacheOrNewResult<T>> {
	const cache = await lcf() as LcfResult;
	let data = null;
	let dataSize = 0;

	if (cache.data && !cache.expired) {
		data = cache.data;
		dataSize = getStringSize(stringify(cache.data));
	} else {
		const actual = await lnf() as LnfResult;

		if (actual.data && !actual.error) {
			data = actual.data;
			dataSize = getStringSize(stringify(data));
			await scf(data);
		} else {
			if (cache.data) {
				data = cache.data;
				dataSize = getStringSize(stringify(data));
			} else {
				onerr(actual.error);
				return { data: null, size: "0KB" };
			}
		}
	}

	const prettySize = (dataSize / 1024).toFixed(1);

	return { data: data, size: `${prettySize}KB` };
}
