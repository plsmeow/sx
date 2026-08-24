import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { logger } from "../utils/logger";
import { Buffer } from "../utils/buffer";

/** Структура данных капчи */
interface Captcha {
	type: "web" | "map";
	data: {
		url: string | null;
		base64: string | null;
	}
}

/** Модуль управления обходом капч */
class CaptchaModule {
	private settings: HTMLElement | null = null;
	private cards: HTMLElement | null = null;
	private wrappers: HTMLElement | null = null;

	/** Метод инициализации функций, связанных с обходом капчи */
	public async init(): Promise<void> {
		this.settings = document.getElementById("anti-captcha-settings");
		this.cards = document.getElementById("anti-captcha-cards");
		this.wrappers = document.getElementById("anti-captcha-wrappers");

		await listen<number[]>("captcha:new-web", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const username = buf.readString();
				const captcha_url = buf.readString();

				this.createCaptchaCard(username, {
					type: "web",
					data: {
						url: captcha_url,
						base64: null
					}
				});
			} catch (error) {
				logger.log(`Ошибка обработки события "captcha:new-web": ${error}`, "error");
			}
		});

		await listen<number[]>("captcha:new-map", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const username = buf.readString();
				const parts = buf.readU8();

				let base64 = "";

				for (let i = 0; i < parts; i++) {
					base64 += buf.readString();
				}

				this.createCaptchaCard(username, {
					type: "map",
					data: {
						url: null,
						base64: base64
					}
				});
			} catch (error) {
				logger.log(`Ошибка обработки события "captcha:new-map": ${error}`, "error");
			}
		});

		await listen<number[]>("captcha:remove", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const username = buf.readString();

				document.getElementById(`bot-captcha-${username}`)?.remove();
				document.getElementById(`captcha-wrapper-${username}`)?.remove();
			} catch (error) {
				logger.log(`Ошибка обработки события "captcha:remove": ${error}`, "error");
			}
		});
	}

	/** Метод активации обхода капчи */
	public enable(type: number, mode: number): void {
		if (type === 0x00 || (type === 0x01 && mode === 0x00)) {
			this.settings!.style.display = "none";

			this.cards!.innerHTML = "";
			this.cards!.style.display = "flex";
		}
	}

	/** Метод очистки и выключения обхода капчи */
	public disable(): void {
		document.querySelectorAll('[wrapper="captcha-card"]').forEach(w => w.remove());
		document.querySelectorAll('[wrapper="captcha-img"]').forEach(w => w.remove());

		this.cards!.innerHTML = "";
		this.cards!.style.display = "none";

		this.wrappers!.innerHTML = "";

		this.settings!.style.display = "flex";
	}

	/** Метод создания карточки капчи */
	private createCaptchaCard(username: string, captcha: Captcha): void {
		const oldCard = document.getElementById(`bot-captcha-${username}`);

		if (oldCard) {
			oldCard.remove();
			document.getElementById(`captcha-wrapper-${username}`)?.remove();
		}

		const card = document.createElement("div");
		card.className = "bot-captcha";
		card.id = `bot-captcha-${username}`;
		card.setAttribute("wrapper", "captcha-card");

		card.innerHTML = `
      <div class="head">
        <div class="username">${username}</div>
      </div>

      <div class="buttons">
        <button class="min" id="solve-captcha-${username}">Решить</button>
        <button class="min" id="remove-captcha-${username}">Удалить</button>
      </div>
    `;

		this.cards?.appendChild(card);

		this.initializeCaptchaCard(username, captcha);
	}

	/** Метод инициализации карточки капчи */
	private initializeCaptchaCard(username: string, captcha: Captcha): void {
		if (captcha.type === "web") {
			document.getElementById(`solve-captcha-${username}`)?.addEventListener("click", async () => {
				const url = captcha.data.url;
				if (url && url !== "") await invoke("open_url", { url: url });
			});
		} else {
			const wrapper = document.createElement("div");
			wrapper.className = "cover";
			wrapper.id = `captcha-wrapper-${username}`;
			wrapper.setAttribute("wrapper", "captcha-img");

			wrapper.innerHTML = `
        <div class="panel with-header" style="margin-bottom: 20px;">
          <div class="left">
            <div class="header">Капча бота ${username}</div>
          </div>

          <div class="right">
            <button class="min" id="close-captcha-wrapper-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M18 6l-12 12" />
                <path d="M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <div class="bot-captcha-wrapper">
          <img class="captcha-img" id="captcha-img-${username}" src="data:image/png;base64,${captcha.data.base64}" draggable="false">
        </div>

        <div class="pretty-input-wrapper" style="margin-top: 20px;">
          <p class="signature">${username}</p>
          <input type="text" id="captcha-code-${username}" placeholder="Введите и отправьте код с капчи, нажав «Enter»">
        </div>
      `;

			this.wrappers?.appendChild(wrapper);

			const img = document.getElementById(`captcha-img-${username}`) as HTMLImageElement;
			img.onload = () => {
				const scale = Math.min(2, 800 / img.naturalWidth);
				img.style.width = (img.naturalWidth * scale) + "px";
				img.style.height = (img.naturalHeight * scale) + "px";
			};

			document.getElementById(`solve-captcha-${username}`)?.addEventListener("click", () => wrapper.style.display = "flex");
			document.getElementById(`close-captcha-wrapper-${username}`)?.addEventListener("click", () => wrapper.style.display = "none");

			document.getElementById(`captcha-wrapper-${username}`)?.addEventListener("keydown", async (e: Event) => {
				if ((e as KeyboardEvent).key === "Enter") {
					const message = document.getElementById(`captcha-code-${username}`) as HTMLInputElement;

					const buf = new Buffer();
					buf.writeU8(0x05);
					buf.writeU8(0x00);
					buf.writeString(username);
					buf.writeString(message.value);

					const result = await invoke<CommandResult<null>>("send_command", {
						data: buf.toUint8Array(),
					});

					if (result.error) {
						logger.log(`Ошибка отправки сообщения от бота ${username}: ${result.error}`, "error");
						return;
					}

					message.value = "";
				}
			});
		}

		document.getElementById(`remove-captcha-${username}`)?.addEventListener("click", async () => {
			const buf = new Buffer();
			buf.writeU8(0x04);
			buf.writeString(username);

			const result = await invoke<CommandResult<null>>("send_command", {
				data: buf.toUint8Array(),
			});

			if (result.error) logger.log(`Ошибка удаления капчи бота ${username}: ${result.error}`, "error");
		});
	}
}

const captchaModule = new CaptchaModule();

export { captchaModule }
