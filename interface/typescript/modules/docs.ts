import { invoke } from "@tauri-apps/api/core";

import { marked } from "marked";
import DOMPurify from "dompurify";

import { logger } from "../utils/logger";
import { loadCacheOrNew, LcfResult, LnfResult, loadCache, saveCache, TimeType } from "../utils/cache";

/** Функция очистки HTML */
const sanitizeHtml = (html: string): string => DOMPurify.sanitize(html, {
	ALLOWED_TAGS: [
		"b", "i", "em", "strong", "small", "mark", "del", "ins", "s", "sub", "sup",
		"u", "span", "abbr", "cite", "dfn", "kbd", "samp", "var", "h1", "h2", "p",
		"h3", "h4", "h5", "h6", "ul", "ol", "li", "dl", "dt", "dd", "table", "thead",
		"tbody", "tfoot", "tr", "th", "td", "caption", "col", "colgroup", "a", "img",
		"pre", "code", "section", "article", "header", "footer", "main", "aside", "nav",
		"figure", "figcaption", "details", "summary", "fieldset", "legend", "label"
	],
	ALLOWED_ATTR: [],
	ALLOW_DATA_ATTR: false,
});

/** Модуль управления пользовательской документацией */
class DocsModule {
	/** Метод инициализации модуля */
	public async init(): Promise<void> {
		try {
			logger.log("Загрузка документации...", "system");

			const docsText = document.querySelector(".documentation-wrapper .text") as HTMLDivElement;
			const docsNavigation = document.querySelector(".documentation-wrapper .navigation") as HTMLDivElement;

			const onerr = (err: string) => logger.log(`Ошибка загрузки документации (не критично): ${err}`, "non-critical-error");
			const result = await loadCacheOrNew<string>(this.lcf, this.lnf, this.scf, onerr);
			if (!result.data) return;

			const html = await marked.parse(result.data);
			docsText.innerHTML = sanitizeHtml(html);

			let navigationOpened = false;
			const headers = document.querySelectorAll<HTMLElement>(".documentation-wrapper .text h1");

			docsNavigation.addEventListener("click", () => {
				if (navigationOpened) return;
				navigationOpened = true;

				docsNavigation.style.width = "auto";
				docsNavigation.style.paddingLeft = "10px";
				docsNavigation.style.paddingRight = "20px";

				headers.forEach((header, i) => {
					const headerTag = document.createElement("label");
					headerTag.className = "header";
					headerTag.innerHTML = `<span>${i + 1}.</span> ${header.innerText}`;
					headerTag.addEventListener("click", () => header.scrollIntoView({ behavior: "smooth", block: "start" }));
					docsNavigation.appendChild(headerTag);

					let nextElement = header.nextElementSibling as HTMLElement;
					let ii = 1;

					while (nextElement && !nextElement.matches("h1")) {
						if (nextElement.matches("h2")) {
							const subheader = nextElement;

							const subheaderTag = document.createElement("label");
							subheaderTag.className = "subheader";
							subheaderTag.innerHTML = `<span>${ii}.</span> ${subheader.innerText}`;
							subheaderTag.addEventListener("click", () => subheader.scrollIntoView({ behavior: "smooth", block: "start" }));
							docsNavigation.appendChild(subheaderTag);

							ii++;
						}

						nextElement = nextElement.nextElementSibling as HTMLElement;
					}
				});
			});

			docsText.addEventListener("click", () => {
				if (!navigationOpened) return;
				navigationOpened = false;

				docsNavigation.innerHTML = "";
				docsNavigation.style.padding = "8px 0";
				docsNavigation.style.width = "18px";
			});

			logger.log(`Документация загружена (размер ${result.size})`, "system");
		} catch (error) {
			logger.log(`Ошибка загрузки документации (не критично): ${error}`, "non-critical-error");
		}
	}

	/** Метод загрузки актуальной пользовательской документации */
	private async lnf(): Promise<LnfResult> {
		try {
			const docs = await invoke<string | null>("download_text", {
				url: "https://codeberg.org/nullclyze/salarixi/raw/branch/main/docs/user.md"
			});

			return { data: docs, error: docs ? null : "resource moved or unavailable" };
		} catch (err) {
			return { data: null, error: String(err) };
		}
	}

	/** Метод загрузки кэшированной пользовательской документации */
	private async lcf(): Promise<LcfResult> {
		const result = await loadCache("user-docs", "user-docs.md", 2, TimeType.Day);
		if (!result.data) return { data: null, incompatible: false, expired: false };

		return {
			data: result.data,
			incompatible: result.error === "incompatible",
			expired: result.error === "expired",
		};
	}

	/** Метод кэширования пользовательской документации */
	private async scf(docs: string): Promise<void> {
		const result = await saveCache(docs, "user-docs.md");

		if (!result.data && result.error) {
			logger.log(`Ошибка кэширования документации (не критично): ${result.error}`, "non-critical-error");
		}
	}
}

const docsModule = new DocsModule();

export { docsModule }
