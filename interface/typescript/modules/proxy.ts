import { invoke } from "@tauri-apps/api/core";
import { logger } from "../utils/logger";

/** Модуль управления прокси и прокси-сборщика */
class ProxyModule {
	private proxyList: HTMLTextAreaElement | null;
	private counter: HTMLElement | null;
	private status: HTMLElement | null;

	constructor() {
		this.proxyList = null;
		this.counter = null;
		this.status = null;
	}

	/** Метод инициализации функций, связанных с сборщиком прокси */
	public async init(): Promise<void> {
		this.proxyList = document.getElementById("proxy-list") as HTMLTextAreaElement;
		this.counter = document.getElementById("proxy-counter") as HTMLElement;
		this.status = document.getElementById("proxy-finder-status") as HTMLElement;

		this.proxyList.addEventListener("input", () => this.updateCounter());

		document.getElementById("clear-proxy-list")?.addEventListener("click", () => {
			this.proxyList!.value = "";
			this.updateCounter();
		});

		document.getElementById("find-proxies")?.addEventListener("click", () => this.collectProxies());
		document.getElementById("check-proxies")?.addEventListener("click", () => this.checkProxies());

		this.updateCounter();
	}

	/** Метод обновления счётчика прокси */
	private updateCounter(value?: string): void {
		if (value) {
			this.counter!.innerText = value;
			return;
		}

		let text = this.proxyList!.value;

		if (!text) {
			this.counter!.innerText = "0";
			return;
		}

		let regex = /(?:\w+:\/\/)?(?:(?:\d{1,3}\.){3}\d{1,3}:\d+)/g;
		let matches = text.match(regex);
		this.counter!.innerText = matches ? matches.length.toString() : "0";
	}

	/** Вспомогательный метод установки статуса поиска прокси */
	private setStatus(text: string, color?: string): void {
		if (!this.status) return;
		this.status!.style.color = color ?? "#848080";
		this.status!.innerText = text;
	}

	/** Метод сборки прокси */
	private async collectProxies(): Promise<void> {
		try {
			this.proxyList!.value = "";

			const algorithm = (document.getElementById("proxy-finder_select_algorithm") as HTMLSelectElement).value;
			const country = (document.getElementById("proxy-finder_select_country") as HTMLSelectElement).value;
			const port = (document.getElementById("proxy-finder_select_port") as HTMLSelectElement).value;
			const count = (document.getElementById("proxy-finder_select_count") as HTMLInputElement).value;

			this.setStatus("Поиск прокси...");

			const bytes = await invoke<number[]>("collect_proxies", {
				options: {
					algorithm: algorithm,
					country: country,
					port: port,
					count: count,
				}
			});

			const uint8arr = new Uint8Array(bytes);
			const decoder = new TextDecoder("utf-8");
			const str = decoder.decode(uint8arr);

			if (str === "") {
				this.setStatus("Ошибка поиска", "#cc1d1dff");
				logger.log("Ошибка сборщика прокси: Could not find a proxies", "error");
				return;
			}

			const proxyCount = str.split("\n").length;

			this.proxyList!.value = str;
			this.updateCounter(proxyCount.toString());
			this.setStatus("Поиск окончен", "#0cd212ff");
		} catch (error) {
			this.setStatus("Ошибка поиска", "#cc1d1dff");
			logger.log(`Ошибка сборщика прокси: ${error}`, "error");
		} finally {
			setTimeout(() => this.setStatus("Поиск неактивен"), 2000);
		}
	}

	/** Метод проверки прокси */
	private async checkProxies(): Promise<void> {
		try {
			const proxies = this.proxyList?.value;
			if (!proxies) return;
			this.proxyList!.value = "";
			this.updateCounter("0");

			const encoder = new TextEncoder();
			const inbytes = encoder.encode(proxies);

			const out = await invoke<number[]>("check_proxies", {
				bytes: Array.from(inbytes),
			});

			const outbytes = new Uint8Array(out);
			const decoder = new TextDecoder("utf-8");
			const str = decoder.decode(outbytes);

			if (str === "") return;

			const proxyCount = str.split("\n").length;

			this.proxyList!.value = str;
			this.updateCounter(proxyCount.toString());
		} catch (error) {
			logger.log(`Ошибка проверки прокси: ${error}`, "error");
		}
	}
}

const proxyModule = new ProxyModule();

export { proxyModule }
