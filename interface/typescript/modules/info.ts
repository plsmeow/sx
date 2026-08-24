import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { addOpeningUrlTo } from "../main";
import { loadCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Структура информации клиента */
interface ClientInfo {
	repo: {
		releases: number;
		stars: number;
		forks: number;
	};
	social: any;
}

/** Небольшой модуль управления информацией клиента */
class InfoModule {
	/** Метод инициализации модуля */
	public async init(): Promise<void> {
		try {
			logger.log("Загрузка информации о клиенте...", "system");

			// Думаю нету смысла хранить актуальную ссылку на репозиторий, так как 
			// если он будет перенесён - загрузить актуальную ссылку не получится
			addOpeningUrlTo("codeberg", "click", "https://codeberg.org/nullclyze/salarixi");

			const releaseCountEl = document.getElementById("release-count") as HTMLSpanElement;
			const starsCountEl = document.getElementById("stars-count") as HTMLSpanElement;
			const forksCountEl = document.getElementById("forks-count") as HTMLSpanElement;

			const onerr = (err: string) => {
				["telegram", "discord", "youtube"].forEach(id => document.getElementById(id)?.remove());
				document.getElementById("support-notice")?.remove();
				logger.log(`Ошибка загрузки информации о клиенте (не критично): ${err}`, "non-critical-error");
			};

			const result = await loadCacheOrNew<ClientInfo>(this.lcf, this.lnf, this.scf, onerr);
			if (!result.data) return;

			releaseCountEl.innerText = result.data.repo.releases > 0 ? String(result.data.repo.releases) : "?";
			starsCountEl.innerText = result.data.repo.stars > 0 ? String(result.data.repo.stars) : "?";
			forksCountEl.innerText = result.data.repo.forks > 0 ? String(result.data.repo.forks) : "?";

			for (const key in result.data.social) {
				const url = result.data.social[key] as string;

				if (key === "donate") {
					const supportNotice = document.getElementById("support-notice");
					if (!supportNotice) continue;
					addOpeningUrlTo("open-donate-page", "click", url);
					document.getElementById("close-support-notice-btn")?.addEventListener("click", () => supportNotice.remove());
					setTimeout(() => document.getElementById("update-notice")?.style.display === "flex" ? supportNotice.remove() : supportNotice.style.display = "flex", 6000);
				} else {
					addOpeningUrlTo(key, "click", url);
				}
			}

			logger.log(`Информация о клиенте загружена (размер ${result.size})`, "system");
		} catch (error) {
			logger.log(`Ошибка инициализации информации о клиенте (не критично): ${error}`, "non-critical-error");
		}
	}

	/** Метод загрузки актуальной информации о клиенте */
	private async lnf(): Promise<LnfResult> {
		try {
			const repository = await invoke<any>("download_json", {
				url: "https://codeberg.org/api/v1/repos/nullclyze/salarixi"
			});

			if (!repository) return { data: null, error: "resource moved or unavailable" };

			const social = await invoke<any>("download_json", {
				url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.social.json"
			});

			if (!social) return { data: null, error: "resource moved or unavailable" };

			return {
				data: {
					repo: {
						releases: repository["release_counter"] ?? repository["release_count"] ?? repository["releases"] ?? 0,
						stars: repository["stars_count"] ?? repository["stars"] ?? 0,
						forks: repository["forks_count"] ?? repository["forks"] ?? 0,
					},
					social: social,
				},
				error: null,
			};
		} catch (err) {
			return { data: null, error: String(err) };
		}
	}

	/** Метод загрузки кэшированной информации о клиенте */
	private async lcf(): Promise<LcfResult> {
		const result = await loadCache("client-info", "client-info.json", 3, TimeType.Day);
		if (!result.data) return { data: null, incompatible: false, expired: false };

		return {
			data: JSON.parse(result.data),
			incompatible: result.error === "incompatible",
			expired: result.error === "expired",
		};
	}

	/** Метод кэширования информации */
	private async scf(info: ClientInfo): Promise<void> {
		const result = await saveCache(JSON.stringify(info, null, 2), "client-info.json");

		if (!result.data && result.error) {
			logger.log(`Ошибка кэширования информации о клиенте (не критично): ${result.error}`, "non-critical-error");
		}
	}
}

const infoModule = new InfoModule();

export { infoModule }
