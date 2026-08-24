import { CLIENT_VERSION } from "../version";

/** Функция сравнения текущей версии с указанной по системе SemVer */
export function isVersionNewer(version: string): boolean {
	const [nmajor, nminor, npatch] = parseVersion(version);
	const [cmajor, cminor, cpatch] = parseVersion(CLIENT_VERSION);

	if (nmajor === -1 || nminor === -1 || npatch === -1 || cmajor === -1 || cminor === -1 || cpatch === -1) return false;

	if (nmajor !== cmajor) return nmajor > cmajor ? true : false;
	if (nminor !== cminor) return nminor > cminor ? true : false;
	if (npatch !== cpatch) return npatch > cpatch ? true : false;

	return false;
}

/** Вспомогательная функция парсинга версии */
function parseVersion(version: string): number[] {
	const split = version.split("-")[0].split(".");
	const major = parseInt(split[0] ?? -1);
	const minor = parseInt(split[1] ?? -1);
	const patch = parseInt(split[2] ?? -1);

	return [major, minor, patch];
}
