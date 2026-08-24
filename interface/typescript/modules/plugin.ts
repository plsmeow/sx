import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { plugins } from "../common/structs";
import { processActivity } from "../main";
import { loadCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Модуль загрузки и управления плагинами */
class PluginModule {
	/** Метод инициализации функций, связанных с плагинами */
	public async init(): Promise<void> {
		this.createCards();
		for (const id in plugins) this.updatePluginState(id, localStorage.getItem(`plugin-state:${id}`) === "true");
		await this.initPluginsInfo();
	}

	/** Метод обновления и сохранения состояния плагина (включен / выключен) */
	private updatePluginState(id: string, state: boolean): void {
		if (processActivity) {
			logger.log(`Не удалось изменить состояние плагина ${id}: Plugin states can only be changed before startup`, "warning");
			return;
		}

		plugins[id].enable = state;

		localStorage.setItem(`plugin-state:${id}`, String(state));

		const toggler = document.getElementById(`${id}-toggler`) as HTMLButtonElement;

		if (state) {
			toggler.setAttribute("state", "false");
			toggler.style.color = "#4ed618";
			toggler.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-check">
          <path stroke="none" d="M0 0h24v24H0z" fill="none" />
          <path d="M5 12l5 5l10 -10" />
        </svg>
      `;
		} else {
			toggler.setAttribute("state", "true");
			toggler.style.color = "#e61919";
			toggler.innerHTML = `
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
          <path stroke="none" d="M0 0h24v24H0z" fill="none" />
          <path d="M18 6l-12 12" />
          <path d="M6 6l12 12" />
        </svg>
      `;
		}
	}

	/** Метод создания и инициализации карточек плагинов */
	private createCards(): void {
		for (const id in plugins) {
			try {
				const plugin = plugins[id];

				const pluginCard = document.createElement("div");
				pluginCard.className = "plugin-card";
				pluginCard.id = `${id}-plugin`;

				pluginCard.innerHTML = `
          <label class="name" id="${id}-name">${plugin.name}</label>

          <div class="controls">
            <button plugin-open-description="true" path="${id}-plugin-description">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-align-justified">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M4 6l16 0" />
                <path d="M4 12l16 0" />
                <path d="M4 18l12 0" />
              </svg>
            </button>

            <button plugin-toggler="true" plugin-id="${id}" state="true" id="${id}-toggler" style="color: #d61818;">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M18 6l-12 12" />
                <path d="M6 6l12 12" />
              </svg>
            </button>
          </div>
        `;

				document.getElementById("plugin-list")?.appendChild(pluginCard);

				const pluginDescription = document.createElement("div");
				pluginDescription.className = "cover";
				pluginDescription.id = `${id}-plugin-description`;

				pluginDescription.innerHTML = `
          <div class="panel with-header">
            <div class="left">
              <div class="header">Описание плагина "${plugin.name}"</div>
            </div>

            <div class="right">
              <button class="min" plugin-close-description="true" path="${id}-plugin-description">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
                  <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                  <path d="M18 6l-12 12" />
                  <path d="M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>

          <div class="plugin-description" id="${id}-description"></div>
          <p class="plugin-latest-update">Последнее обновление плагина: <span id="${id}-latest-update">?</span></p>
        `;

				document.getElementById("plugins-container")?.appendChild(pluginDescription);
			} catch (error) {
				logger.log(`Ошибка инициализации плагина ${id}: ${error}`, "error");
			}
		}

		document.querySelectorAll('[plugin-toggler="true"]').forEach(t => t.addEventListener("click", () => {
			const id = t.getAttribute("plugin-id") || "";
			const state = t.getAttribute("state") === "true";
			this.updatePluginState(id, state);
		}));

		document.querySelectorAll('[plugin-open-description="true"]').forEach(e => e.addEventListener("click", () => {
			const path = e.getAttribute("path");
			if (!path) return;
			(document.getElementById(path) as HTMLElement).style.display = "flex";
		}));

		document.querySelectorAll('[plugin-close-description="true"]').forEach(e => e.addEventListener("click", () => {
			const path = e.getAttribute("path");
			if (!path) return;
			(document.getElementById(path) as HTMLElement).style.display = "none";
		}));
	}

	/** Метод инициализации информации о плагинах */
	private async initPluginsInfo(): Promise<void> {
		try {
			logger.log("Загрузка информации о плагинах...", "system");

			const onerr = (err: string) => logger.log(`Ошибка загрузки информации о плагинах (не критично): ${err}`, "non-critical-error");
			const result = await loadCacheOrNew<any>(this.lcf, this.lnf, this.scf, onerr);
			if (!result.data) return;

			const data = result.data;
			const pluginList = data["list"] ?? data["array"] ?? data["map"] ?? data;

			// Может быть такое что названия полей изменились, или вовсе вся структура
			// данных обновлена, поэтому здесь стоит делать ещё одну проверку
			if (!pluginList) {
				logger.log("Ошибка загрузки информации о плагинах (не критично): information is corrupted or missing", "non-critical-error");
				return;
			}

			this.extractPluginsInfo(pluginList);

			logger.log(`Информация о плагинах загружена (размер ${result.size})`, "system");
		} catch (error) {
			logger.log(`Ошибка загрузки информации о плагинах (не критично): ${error}`, "non-critical-error");
		}
	}

	/** Метод извлечения информации плагинов */
	private extractPluginsInfo(pluginList: any): void {
		for (const plugin of pluginList) {
			const id = plugin["id"] ?? plugin["name"] ?? plugin["header"] ?? plugin["kind"];
			const desc = plugin["description"] ?? plugin["purpose"];
			const updated = plugin["latest_update"] ?? plugin["updated"] ?? plugin["modified"];

			if (plugins[id]?.date) {
				if (updated !== plugins[id].date) {
					document.getElementById(`${id}-plugin`)?.classList.add("deprecated");

					const tag = document.createElement("span");
					tag.className = "tag";
					tag.innerText = "deprecated";

					document.getElementById(`${id}-name`)?.appendChild(tag);
				}

				const pluginDescriptionContainer = document.getElementById(`${id}-description`) as HTMLElement;
				const pluginLatestUpdateContainer = document.getElementById(`${id}-latest-update`) as HTMLElement;

				for (const p of desc) {
					const el = document.createElement("p");
					el.innerText = p;
					pluginDescriptionContainer.appendChild(el);
				}

				pluginLatestUpdateContainer.innerText = updated;
			}
		}
	}

	/** Метод загрузки актуальной информации о плагинах */
	private async lnf(): Promise<LnfResult> {
		try {
			const plugins = await invoke<any>("download_json", {
				url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/salarixi.plugins.json"
			});

			return { data: plugins, error: plugins ? null : "resource moved or unavailable" };
		} catch (err) {
			return { data: null, error: String(err) };
		}
	}

	/** Метод загрузки кэшированной информации о плагинах */
	private async lcf(): Promise<LcfResult> {
		const result = await loadCache("plugins-info", "plugins-info.json", 4, TimeType.Day);
		if (!result.data) return { data: null, incompatible: false, expired: false };

		return {
			data: JSON.parse(result.data),
			incompatible: result.error === "incompatible",
			expired: result.error === "expired",
		};
	}

	/** Метод кэширования информации о плагинах */
	private async scf(info: string): Promise<void> {
		const result = await saveCache(JSON.stringify(info, null, 2), "plugins-info.json");

		if (!result.data && result.error) {
			logger.log(`Ошибка кэширования информации о плагинах (не критично): ${result.error}`, "non-critical-error");
		}
	}
}

const pluginModule = new PluginModule();

export { pluginModule }
