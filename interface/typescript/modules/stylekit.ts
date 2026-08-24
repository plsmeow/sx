import { open } from "@tauri-apps/plugin-dialog";
import { logger } from "../utils/logger";
import { messages } from "../utils/message";
import { invoke } from "@tauri-apps/api/core";

/** Структура результата применения набора стилей */
interface ApplyResult {
	total: number;
	coincidences: number;
}

/** Модуль управления наборами стилей */
class StylekitModule {
	private defaultStyleKit: Record<string, string> = {
		"title-color": "linear-gradient(to right, #c5c5c5 8%, #7e47ff, #5096e6, #7e47ff)",
		"highlight-color": "#7946f0",
		"background": "#0e0e0e",
		"primary-bg": "#0e0e0e",
		"primary-border": "#333333",
		"secondary-bg": "#111111",
		"secondary-border": "#2c2c2c",
		"bright-bg": "#1a1a1a",
		"bright-border": "#474747",
		"hover-bg": "#3b3b3b50",
		"hover-border": "#555555",
		"chart-bar-bg": "#13812b",
		"process-inactive-color": "#d1d415",
		"process-active-color": "#13d40c",
		"process-launching-color": "#858585",
		"process-stopping-color": "#858585",
		"cover-bg": "#111111",
		"cover-border": "#292929",
		"cover-bg-blur": "0",
		"default-color": "#dfdfdf",
		"muted-color": "#b6b6b6",
		"dim-color": "#808080",
		"selection-color": "#ffffff1f",
		"info-text-color": "#747474",
		"input-placeholder": "#888888",
		"input-border-radius": "10px",
		"input-bg": "#141414",
		"input-border": "#2c2c2c",
		"checkbox-mark-color": "#a7a7a7",
		"checkbox-mark-border-radius": "6px",
		"checkbox-bg": "#1b1b1b",
		"checkbox-border": "#313131",
		"checkbox-hover-bg": "#242424",
		"checkbox-hover-border": "#424242",
		"select-mark-color": "#c5c5c5",
		"select-border-radius": "10px",
		"select-bg": "#141414",
		"select-border": "#2c2c2c",
		"select-hover-bg": "#141414",
		"select-hover-border": "#363636",
		"dashboard-button-border-radius": "4px",
		"dashboard-button-bg": "transparent",
		"dashboard-button-border": "transparent",
		"dashboard-button-color": "#b6b6b6",
		"dashboard-button-hover-bg": "#1f1f1f",
		"dashboard-button-hover-border": "transparent",
		"dashboard-button-hover-color": "#dfdfdf",
		"dashboard-button-selected-bg": "#1f1f1f",
		"dashboard-button-selected-border": "transparent",
		"dashboard-button-selected-color": "#dfdfdf",
		"button-border-radius": "10px",
		"button-bg": "#141414",
		"button-border": "#2c2c2c",
		"button-color": "#dfdfdf",
		"button-hover-bg": "#1f1f1f",
		"button-hover-border": "#424242",
		"button-hover-color": "#dfdfdf",
		"button-active-bg": "#141414",
		"button-active-border": "#303030",
		"button-active-color": "#b6b6b6",
		"info-log-color": "#cecfcf",
		"warning-log-color": "#f38c2f",
		"error-log-color": "#ff6b6b",
		"non-critical-error-log-color": "#e07e7e",
		"system-log-color": "#707070",
		"extended-log-color": "#919191",
		"message-progress-color": "#7946f0",
		"message-bg": "#111111cc",
		"message-border": "#2c2c2c",
		"message-border-radius": "8px",
		"bot-connected-status-color": "#22ed17",
		"bot-disconnected-status-color": "#ed1717",
		"bot-waiting-status-color": "#8f8f8f",
		"plugin-tag-border-radius": "8px",
		"plugin-deprecated-tag-color": "#d3c017",
		"plugin-deprecated-tag-bg": "#f7e22013",
		"plugin-deprecated-tag-border": "#f7e1207a",
		"docs-header-color": "#dfdfdf",
		"docs-code-bg": "#3a393952",
		"docs-code-border": "#3a3939c2",
		"docs-navigation-bg": "#111111",
		"docs-navigation-border": "#2c2c2c",
		"update-notice-warning-color": "#ebbf31",
		"update-notice-warning-bg": "#ebbf310d",
		"update-notice-warning-border": "#ebc0312c",
		"support-notice-text-bg": "#1383042e",
		"support-notice-text-border": "#0a4d01",
		"scrollbar-bg": "#161616",
		"scrollbar-thumb-bg": "#353535",
		"scrollbar-thumb-hover-bg": "#5e5e5e",
	};

	/** Метод инициализации модуля */
	public init(): void {
		this.loadSaved();

		document.getElementById("select-stylekit")?.addEventListener("click", async () => await this.select());
		document.getElementById("reset-stylekit")?.addEventListener("click", () => this.reset());
	}

	/** Метод выбора набора стилей */
	private async select(): Promise<void> {
		try {
			const path = await open({
				directory: false,
				multiple: false,
				filters: [{
					name: "Style Kit",
					extensions: ["json"],
				}],
			});

			if (!path) return;

			const name = path.split(/[\\/]/).pop();

			if (!name) {
				messages.message("Набор стилей", "Не удалось применить набор стилей");
				logger.log("Ошибка применения набора стилей: Failed to get file name", "error");
				return;
			}

			const kitBytes = await invoke<number[]>("read_text_file", { path: path });

			if (kitBytes.length < 1) {
				messages.message("Набор стилей", "Не удалось применить набор стилей");
				logger.log("Ошибка применения набора стилей: Incorrect kit size", "error");
				return;
			}

			const uint8arr = new Uint8Array(kitBytes);
			const decoder = new TextDecoder("utf-8");
			const kit = decoder.decode(uint8arr);
			const kitJson = JSON.parse(kit);

			if (!kitJson) {
				messages.message("Набор стилей", "Не удалось применить набор стилей");
				logger.log("Ошибка применения набора стилей: Failed to parse JSON", "error");
				return;
			}

			const result = this.apply(kitJson);

			if (result.coincidences < 1) {
				messages.message("Набор стилей", "Не удалось применить набор стилей");
				logger.log("Ошибка применения набора стилей: Number of matches is less than 1", "error");
				return;
			}

			this.setStylekitName(name);

			localStorage.setItem("stylekit", JSON.stringify({
				name: name,
				kit: kitJson,
			}));

			messages.message("Набор стилей", `Набор стилей "${name}" применён`);
			logger.log(`Набор стилей "${name}" применён (${result.coincidences} из ${result.total} совпадений)`, "system");
		} catch (error) {
			logger.log(`Ошибка применения набора стилей: ${error}`, "error");
		}
	}

	/** Метод загрузки сохранённого набора стилей */
	private loadSaved(): void {
		const stylekit = localStorage.getItem("stylekit");
		if (!stylekit || stylekit.length < 1) return;

		const stylekitJson = JSON.parse(stylekit);
		const name = stylekitJson["name"] ?? "unknown";
		const kit = stylekitJson["kit"] ?? {};
		const result = this.apply(kit);

		if (result.coincidences < 1) return;

		this.setStylekitName(name);

		logger.log(`Сохранённый набор стилей "${name}" загружен (${result.coincidences} из ${result.total} совпадений)`, "system");
	}

	/** Метод установки имени набора стилей */
	private setStylekitName(name: string): void {
		const currentStylekit = document.getElementById("current-stylekit");
		if (!currentStylekit) return;
		currentStylekit.innerText = name;
	}

	/** Метод применения набора стилей */
	private apply(kit: any): ApplyResult {
		const root = document.documentElement;
		let total = 0;
		let coincidences = 0;

		for (const key in this.defaultStyleKit) {
			total++;

			const fullKey = `--${key}`;
			let customValue = kit[key];

			if (customValue) {
				coincidences++;
				root.style.setProperty(fullKey, customValue);
				continue;
			}

			customValue = kit[fullKey];

			if (customValue) {
				coincidences++;
				root.style.setProperty(fullKey, customValue);
				continue;
			}

			root.style.setProperty(fullKey, this.defaultStyleKit[key]);
		}

		return {
			total: total,
			coincidences: coincidences,
		};
	}

	/** Метод сброса набора стилей */
	private reset(): void {
		localStorage.removeItem("stylekit");

		const root = document.documentElement;

		for (const key in this.defaultStyleKit) {
			const defaultValue = this.defaultStyleKit[key];
			root.style.setProperty(`--${key}`, defaultValue);
		}

		this.setStylekitName("DEFAULT");

		messages.message("Набор стилей", "Набор стилей сброшен");
	}
}

const stylekitModule = new StylekitModule();

export { stylekitModule }
