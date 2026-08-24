import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Chart, registerables } from "chart.js";

import { plugins } from "./common/structs";
import { logger } from "./utils/logger";
import { messages } from "./utils/message";
import { enableParticles } from "./effects/particles";
import { switchControlWrapper, switchGlobalWrapper } from "./utils/switchers";
import { replenishTriggerRegistry } from "./trigger/replenish";
import { Buffer } from "./utils/buffer";

import { configModule } from "./modules/config";
import { accountModule } from "./modules/account";
import { proxyModule } from "./modules/proxy";
import { chartModule } from "./modules/chart";
import { scriptModule } from "./modules/script";
import { monitoringModule } from "./modules/monitoring";
import { captchaModule } from "./modules/captcha";
import { radarModule } from "./modules/radar";
import { pingModule } from "./modules/ping";
import { translatorModule } from "./modules/translator";
import { pluginModule } from "./modules/plugin";
import { timerModule } from "./modules/timer";
import { docsModule } from "./modules/docs";
import { infoModule } from "./modules/info";
import { updaterModule } from "./modules/updater";
import { stylekitModule } from "./modules/stylekit";
import { scannerModule } from "./modules/scanner";

import { CLIENT_VERSION } from "./version";
import { sessionModule } from "./modules/session";

Chart.register(...registerables);

Chart.defaults.font.family = "'Inter', sans-serif";
Chart.defaults.font.size = 13;
Chart.defaults.font.weight = "normal";

export let processActivity: boolean = false;
export let quickTasksAllowed: boolean = true;

const pressedKeys: { [x: string]: boolean } = {
	alt: false,
	shift: false,
	f: false,
	i: false,
	c: false,
	j: false,
	w: false,
	a: false,
	s: false,
	d: false,
	q: false,
	h: false,
	u: false,
	l: false,
	p: false,
	g: false,
	r: false,
	t: false,
	z: false
};

export let globalWrappers: Array<{ id: string; el: HTMLElement }> = [];
export let controlWrappers: Array<{ id: string; el: HTMLElement }> = [];
export let latestControlWrapper: HTMLElement | null = null;

/** Функция обновления отображаемого статуса процесса */
function updateProcessStatus(statusId: number, connectedBots: number, totalBots: number): void {
	const processStatus = document.getElementById("process-status");
	if (!processStatus) return;

	switch (statusId) {
		case 0x00:
			processStatus.innerText = `stopping ${connectedBots} bots...`;
			processStatus.style.color = "var(--process-stopping-color)";
			break;
		case 0x01:
			processStatus.innerText = `active (${connectedBots}/${totalBots} bots)`;
			processStatus.style.color = "var(--process-active-color)";
			break;
		case 0x02:
			processStatus.innerText = `launching (${connectedBots}/${totalBots} bots)...`;
			processStatus.style.color = "var(--process-launching-color)";
			break;
		case 0x03:
			processStatus.innerText = "inactive";
			processStatus.style.color = "var(--process-inactive-color)";
			break;
	}
}

/** Функция изменения значения `quickTasksAllowed` */
export function setQuickTasksAllowed(value: boolean): void {
	quickTasksAllowed = value;
}

/** Функция запуска ботов */
async function startBots(): Promise<void> {
	try {
		const options: {
			basic: any;
			accounts: Map<string, any>;
			plugins: any;
			captcha_bypass: any;
			webhook: any;
		} = {
			basic: {},
			accounts: new Map(),
			plugins: {},
			captcha_bypass: {},
			webhook: {},
		};

		document.querySelectorAll<HTMLElement>("[launch-option]").forEach(o => {
			const optionSection = o.getAttribute("section") || "basic";
			const optionKey = o.getAttribute("key") || "";

			const current = options[optionSection as "basic" | "plugins" | "captcha_bypass" | "webhook"];

			if (o.tagName.toLocaleLowerCase() === "input") {
				const input = o as HTMLInputElement;

				if (input.type === "checkbox") {
					current[optionKey] = input.checked;
				} else {
					const defaultValue = input.getAttribute("default");

					if (input.type === "number") {
						current[optionKey] = input.value ? parseInt(input.value) : defaultValue ? parseInt(defaultValue!) : null;
					} else {
						current[optionKey] = input.value || defaultValue;
					}
				}
			} else if (o.tagName.toLocaleLowerCase() === "select") {
				const select = o as HTMLSelectElement;
				current[optionKey] = select.selectedIndex;
			} else if (o.tagName.toLocaleLowerCase() === "textarea") {
				const textarea = o as HTMLTextAreaElement;
				current[optionKey] = textarea.value;
			}
		});

		options.accounts = accountModule.getSelectedAccounts();
		options.basic["script"] = options.basic.use_auto_script ? (document.getElementById("user-script") as HTMLTextAreaElement).value : null;

		for (const name in plugins) options.plugins[name.replaceAll("-", "_")] = plugins[name].enable;

		const buf = new Buffer();
		buf.writeU8(0x01);

		buf.writeString(options.basic["address"]);
		buf.writeU8(options.basic["bots_count"]);
		buf.writeU32(options.basic["join_delay"]);
		buf.writeU8(options.basic["nickname_type"]);
		buf.writeU8(options.basic["password_type"]);
		buf.writeU8(options.basic["email_type"]);
		buf.writeString(options.basic["nickname_template"]);
		buf.writeString(options.basic["password_template"]);
		buf.writeU8(options.basic["register_mode"]);
		buf.writeString(options.basic["register_command"]);
		buf.writeString(options.basic["register_template"]);
		buf.writeU32(options.basic["register_min_delay"]);
		buf.writeU32(options.basic["register_max_delay"]);
		buf.writeString(options.basic["register_trigger"]);
		buf.writeU8(options.basic["login_mode"]);
		buf.writeString(options.basic["login_command"]);
		buf.writeString(options.basic["login_template"]);
		buf.writeU32(options.basic["login_min_delay"]);
		buf.writeU32(options.basic["login_max_delay"]);
		buf.writeString(options.basic["login_trigger"]);
		buf.writeU32(options.basic["rejoin_delay"]);
		buf.writeU32(options.basic["monitoring_update_rate"]);
		buf.writeU8(options.basic["view_distance"]);
		buf.writeOption<string>(options.basic["humanoid_arm"], (some) => buf.writeString(some));
		const targetVersionSelect = document.getElementById("settings_select_target-version") as HTMLSelectElement;
		buf.writeOption<string>(targetVersionSelect.value || null, (some) => buf.writeString(some));
		buf.writeBoolean(options.basic["use_auto_rejoin"]);
		buf.writeBoolean(options.basic["use_auto_register"]);
		buf.writeBoolean(options.basic["use_double_auth"]);
		buf.writeBoolean(options.basic["use_auto_login"]);
		buf.writeBoolean(options.basic["use_auto_respawn"]);
		buf.writeBoolean(options.basic["use_accept_rp"]);
		buf.writeBoolean(options.basic["use_pathfinder"]);
		buf.writeBoolean(options.basic["use_auto_script"]);
		buf.writeBoolean(options.basic["use_proxy"]);
		buf.writeBoolean(options.basic["use_accounts"]);
		buf.writeBoolean(options.basic["use_anti_captcha"]);
		buf.writeBoolean(options.basic["use_webhook"]);
		buf.writeBoolean(options.basic["monitoring_optimization"]);
		buf.writeOption<string>(options.basic["proxy_list"], (some) => buf.writeString(some));
		buf.writeOption<string>(options.basic["script"], (some) => buf.writeString(some));

		buf.writeU8(options.accounts.size);
		for (const [username, account] of options.accounts) {
			buf.writeString(username);
			buf.writeOption(account["initial_group"], (some) => buf.writeString(some));
			buf.writeOption(account["password"], (some) => buf.writeString(some));
			buf.writeOption(account["email"], (some) => buf.writeString(some));
			buf.writeOption(account["proxy"], (some) => buf.writeString(some));
		}

		buf.writeBoolean(options.plugins["instant_armor_equip"]);
		buf.writeBoolean(options.plugins["auto_totem"]);
		buf.writeBoolean(options.plugins["auto_eat"]);
		buf.writeBoolean(options.plugins["potion_consumer"]);
		buf.writeBoolean(options.plugins["auto_look"]);
		buf.writeBoolean(options.plugins["auto_shield"]);
		buf.writeBoolean(options.plugins["auto_mending"]);
		buf.writeBoolean(options.plugins["pearl_leave"]);

		buf.writeU8(options.captcha_bypass["captcha_type"]);
		buf.writeU8(options.captcha_bypass["captcha_subtype"]);
		buf.writeU8(options.captcha_bypass["solve_mode"]);
		buf.writeU8(options.captcha_bypass["captcha_size"]);
		buf.writeString(options.captcha_bypass["regex"]);
		buf.writeOption<string>(options.captcha_bypass["required_url_part"], (some) => buf.writeString(some));
		buf.writeU32(options.captcha_bypass["number_of_columns"]);
		buf.writeU32(options.captcha_bypass["number_of_rows"]);
		buf.writeU32(options.captcha_bypass["max_pause"]);
		buf.writeOption<string>(options.captcha_bypass["user_id"], (some) => buf.writeString(some));
		buf.writeOption<string>(options.captcha_bypass["api_key"], (some) => buf.writeString(some));
		buf.writeU8(options.captcha_bypass["api_service"]);
		buf.writeOption<string>(options.captcha_bypass["custom_api_url"], (some) => buf.writeString(some));

		buf.writeOption<string>(options.webhook["url"], (some) => buf.writeString(some));
		buf.writeBoolean(options.plugins["send_information"]);
		buf.writeBoolean(options.plugins["send_data"]);
		buf.writeBoolean(options.plugins["send_actions"]);

		const result = await invoke<CommandResult<null>>("send_command", {
			data: buf.toUint8Array(),
		});

		if (result.error) logger.log(`Ошибка запуска ботов: ${result.error}`, "error");

		processActivity = true;

		monitoringModule.maxChatHistoryLength = parseInt((document.getElementById("monitoring_option_chat-history-length") as HTMLInputElement).value || "50");
		monitoringModule.enable();

		if (options.basic["use_anti_captcha"]) captchaModule.enable(options.captcha_bypass["captcha_type"], options.captcha_bypass["solve_mode"]);

		radarModule.enable();
	} catch (error) {
		logger.log(`Ошибка запуска ботов: ${error}`, "error");
	}
}

/** Функция остановки ботов */
async function stopBots(): Promise<void> {
	try {
		const buf = new Buffer();
		buf.writeU8(0x02);

		const result = await invoke<CommandResult<null>>("send_command", {
			data: buf.toUint8Array(),
		});

		if (result.error) logger.log(`Ошибка остановки ботов: ${result.error}`, "error");
	} catch (error) {
		logger.log(`Ошибка остановки ботов: ${error}`, "error");
	}
}

/** Функция обновления состояния указанного модуля управления */
async function updateModuleState(index: number, state: number): Promise<void> {
	try {
		const buf = new Buffer();
		buf.writeU8(0x07);
		buf.writeU8(index);

		const group = (document.getElementById("control-group") as HTMLInputElement).value.replace(" ", "");
		buf.writeString(group !== "" ? group : "global");

		const elements = document.querySelectorAll(`[module-index="${index}"]`);
		let filteredElements: Map<number, HTMLSelectElement | HTMLInputElement> = new Map();

		elements.forEach(e => {
			const tag = e.tagName.toLowerCase();
			if (tag === "button") return;

			const ordinal = e.getAttribute("ordinal");
			if (!ordinal) return;

			filteredElements.set(parseInt(ordinal), e as any);
		});

		let orderedElements: Array<HTMLSelectElement | HTMLInputElement> = [];

		for (let i = 0; i < filteredElements.size; i++) {
			const el = filteredElements.get(i);
			if (!el) continue;
			orderedElements.push(el);
		}

		orderedElements.forEach(e => {
			const tag = e.tagName.toLowerCase();

			if (tag === "select") {
				// Пусть u8, пока нету селекта, который хранит больше 255 опций
				buf.writeU8((e as HTMLSelectElement).selectedIndex);
			} else {
				const input = e as HTMLInputElement;

				if (e.type === "checkbox") {
					buf.writeU8(input.checked ? 1 : 0);
				} else {
					const optional = input.getAttribute("optional") === "true";

					if (optional) {
						if (input.value === "") {
							buf.writeU8(0);
							return;
						} else {
							buf.writeU8(1);
						}
					}

					if (e.type === "number") {
						const ty = e.getAttribute("num-type");
						if (!ty) return;

						const value = Number(input.value);

						switch (ty) {
							case "u8":
								buf.writeU8(value);
								break;
							case "u16":
								buf.writeU16(value);
								break;
							case "u64":
								buf.writeU64(BigInt(value));
								break;
							case "i32":
								buf.writeI32(value);
								break;
							case "i64":
								buf.writeI64(BigInt(value));
								break;
							case "f32":
								buf.writeF32(value);
								break;
							case "f64":
								buf.writeF64(value);
								break;
						}
					} else {
						buf.writeString(input.value);
					}
				}
			}
		});

		// Пока так, состояние это последняя опция, иначе логически не поставить
		buf.writeU8(state);

		const result = await invoke<CommandResult<null>>("send_command", {
			data: buf.toUint8Array(),
		});

		if (result.error) logger.log(`Ошибка изменения состояния модуля (index=${index}): ${result.error}`, "error");
	} catch (error) {
		logger.log(`Ошибка изменения состояния модуля (index=${index}): ${error}`, "error");
	}
}

/** Функция инициализации глобальных элементов */
function initGlobalElements(): void {
	const startBotsProcessBtn = document.getElementById("start") as HTMLButtonElement;
	const stopBotsProcessBtn = document.getElementById("stop") as HTMLButtonElement;
	const setRandomValuesBtn = document.getElementById("random") as HTMLButtonElement;
	const clearInputValuesBtn = document.getElementById("clear") as HTMLButtonElement;

	const dashboardBtns = document.querySelectorAll<HTMLButtonElement>(".dashboard button");
	const controlBtns = document.querySelectorAll<HTMLButtonElement>(".control-dashboard .button-list button");

	dashboardBtns.forEach(btn => {
		if (btn.id === "main") btn.classList.add("selected");
		btn.addEventListener("click", () => {
			const path = btn.getAttribute("path");
			if (!path) return;
			switchGlobalWrapper(path);
			dashboardBtns.forEach(b => b.classList.remove("selected"));
			btn.classList.add("selected");
		});
	});

	controlBtns.forEach(btn => {
		if (btn.id === "control-chat") btn.classList.add("selected");
		btn.addEventListener("click", () => {
			const path = btn.getAttribute("path");
			if (!path) return;
			switchControlWrapper(path);
			controlBtns.forEach(b => b.classList.remove("selected"));
			btn.classList.add("selected");
			latestControlWrapper = document.getElementById(path);
		});
	});

	document.querySelectorAll<HTMLButtonElement>('[module-toggler]').forEach(e => e.addEventListener("click", async () => await updateModuleState(Number(e.getAttribute("module-index")), Number(e.getAttribute("state")))));

	document.addEventListener("keydown", async (e) => {
		if (!processActivity || !quickTasksAllowed) return;
		const key = e.key.toLowerCase();
		for (const k in pressedKeys) key === k ? pressedKeys[key] = true : null;

		let taskId = undefined;

		if (pressedKeys.shift && pressedKeys.i && pressedKeys.c) {
			taskId = 0x00;
		} else if (pressedKeys.shift && pressedKeys.f && pressedKeys.c) {
			taskId = 0x0E;
		} else if (pressedKeys.shift && pressedKeys.f && pressedKeys.l) {
			taskId = 0x0F;
		} else if (pressedKeys.shift && pressedKeys.p && pressedKeys.s) {
			taskId = 0x10;
		} else if (pressedKeys.shift && pressedKeys.g && pressedKeys.f) {
			taskId = 0x07;
		} else if (pressedKeys.shift && pressedKeys.g && pressedKeys.r) {
			taskId = 0x09;
		} else if (pressedKeys.shift && pressedKeys.g && pressedKeys.c) {
			taskId = 0x0A;
		} else if (pressedKeys.alt && pressedKeys.shift && pressedKeys.q) {
			taskId = 0x08;
		} else if (pressedKeys.shift && pressedKeys.w) {
			taskId = 0x01;
		} else if (pressedKeys.shift && pressedKeys.s) {
			taskId = 0x02;
		} else if (pressedKeys.shift && pressedKeys.a) {
			taskId = 0x03;
		} else if (pressedKeys.shift && pressedKeys.d) {
			taskId = 0x04;
		} else if (pressedKeys.shift && pressedKeys.j) {
			taskId = 0x05;
		} else if (pressedKeys.shift && pressedKeys.c) {
			taskId = 0x06;
		} else if (pressedKeys.shift && pressedKeys.u) {
			taskId = 0x0B;
		} else if (pressedKeys.shift && pressedKeys.t) {
			taskId = 0x0C;
		} else if (pressedKeys.shift && pressedKeys.z) {
			taskId = 0x0D;
		}

		if (taskId !== undefined) {
			const buf = new Buffer();
			buf.writeU8(0x0B);
			buf.writeU8(taskId);

			const result = await invoke<CommandResult<null>>("send_command", {
				data: buf.toUint8Array(),
			});

			if (result.error) logger.log(`Ошибка выполнения быстрой задачи (id=${taskId}): ${result.error}`, "info");
		}
	});

	document.addEventListener("keyup", (e) => {
		if (!processActivity) return;
		const key = e.key.toLowerCase();
		for (const k in pressedKeys) key === k ? pressedKeys[key] = false : null;
	});

	startBotsProcessBtn.addEventListener("click", async () => await startBots());
	stopBotsProcessBtn.addEventListener("click", async () => await stopBots());

	setRandomValuesBtn.addEventListener("click", () => {
		const gen = (e: string): void => {
			let current = document.getElementById(e) as HTMLInputElement;
			switch (e) {
				case "settings_option_bots-count":
					const randomQuantity = Math.floor(Math.random() * (50 - 10 + 1) + 10);
					current.valueAsNumber = randomQuantity; break;
				case "settings_option_join-delay":
					const randomDelay = Math.floor(Math.random() * (7000 - 1000 + 1) + 1000);
					current.valueAsNumber = randomDelay; break;
			}
		}

		gen("settings_option_bots-count");
		gen("settings_option_join-delay");
	});

	clearInputValuesBtn.addEventListener("click", () => {
		(document.getElementById("settings_option_address") as HTMLInputElement).value = "";
		(document.getElementById("settings_option_bots-count") as HTMLInputElement).value = "";
		(document.getElementById("settings_option_join-delay") as HTMLInputElement).value = "";
	});

	const clientSettingsContainer = document.querySelector<HTMLDivElement>(".client-settings-wrapper");
	if (!clientSettingsContainer) return;

	document.getElementById("open-client-settings")?.addEventListener("click", () => clientSettingsContainer.style.display = "flex");
	document.getElementById("close-client-settings")?.addEventListener("click", () => clientSettingsContainer.style.display = "none");
}

/** Функция добавления открывающейся ссылки для определённого события элемента */
export function addOpeningUrlTo(id: string, event: string, url: string): void {
	const el = document.getElementById(id);
	if (!el) return;
	el.addEventListener(event, async () => {
		try {
			await invoke("open_url", { url: url });
		} catch (error) {
			logger.log(`Ошибка открытия URL: ${error}`, "error");
		}
	});
}

/** Вспомогательная функция инициализации титул бара */
function initTitleBar(): void {
	(document.getElementById("window-minimize") as HTMLButtonElement).addEventListener("click", async () => await getCurrentWindow().minimize());
	(document.getElementById("window-close") as HTMLButtonElement).addEventListener("click", async () => {
		await configModule.saveCurrentConfig();
		await invoke("exit");
	});
}

/** Функция инициализации слушателей системных событий */
async function listenSystemEvents(): Promise<void> {
	await listen<number[]>("system:log", (e) => {
		try {
			const buf = new Buffer(new Uint8Array(e.payload));
			const text = buf.readString();
			const type = buf.readString();
			logger.log(text, type);
		} catch (error) {
			logger.log(`Ошибка обработки события "system:log": ${error}`, "error");
		}
	});

	await listen<number[]>("system:message", (e) => {
		try {
			const buf = new Buffer(new Uint8Array(e.payload));
			const name = buf.readString();
			const content = buf.readString();
			messages.message(name, content);
		} catch (error) {
			logger.log(`Ошибка обработки события "system:message": ${error}`, "error");
		}
	});

	await listen<number[]>("status:launch", (e) => {
		try {
			const buf = new Buffer(new Uint8Array(e.payload));
			const status = buf.readU8();

			if (status !== 0x01) {
				processActivity = false;
				monitoringModule.disable();
				captchaModule.disable();
				radarModule.disable();

				switch (status) {
					case 0x00:
						logger.log("Ошибка запуска ботов: protocol or kernel error", "error");
						break;
					case 0x02:
						logger.log("Запуск ботов невозможен: process is already active", "warning");
						break;
					case 0x03:
						logger.log("Ошибка запуска ботов: number of launching bots must be in range 1-255", "error");
						break;
				}
			}
		} catch (error) {
			logger.log(`Ошибка обработки события "status:launch": ${error}`, "error");
		}
	});

	await listen<number[]>("status:stop", (e) => {
		try {
			const buf = new Buffer(new Uint8Array(e.payload));
			const status = buf.readU8();

			if (status === 0x01) {
				monitoringModule.disable();
				captchaModule.disable();
				radarModule.disable();

				processActivity = false;

				messages.message("Система", "Остановка ботов завершена");
				logger.log("Остановка ботов завершена", "info");
			} else {
				switch (status) {
					case 0x00:
						logger.log("Ошибка остановки ботов: protocol or kernel error", "error");
						break;
					case 0x02:
						logger.log("Остановка ботов невозможна: process is already inactive", "warning");
						break;
					case 0x03:
						logger.log("Остановка ботов невозможна: stop is already in progress", "warning");
						break;
				}
			}
		} catch (error) {
			logger.log(`Ошибка обработки события "status:stop": ${error}`, "error");
		}
	});

	await listen<number[]>("process:display-status", (e) => {
		try {
			const buf = new Buffer(new Uint8Array(e.payload));
			const statusId = buf.readU8();
			const connectedBots = buf.readU8();
			const totalBots = buf.readU8();
			updateProcessStatus(statusId, connectedBots, totalBots);
		} catch (error) {
			logger.log(`Ошибка обработки события "process:display-status": ${error}`, "error");
		}
	});

	await listen<number[]>("process:synchronize", (e) => {
		try {
			const buf = new Buffer(new Uint8Array(e.payload));
			const activity = buf.readU8() === 0x01;
			const statusId = buf.readU8();
			const connectedBots = buf.readU8();
			const totalBots = buf.readU8();
			const antiCaptchaEnabled = buf.readU8() === 0x01;
			const captchaType = buf.readU8();
			const captchaSolveMode = buf.readU8();

			processActivity = activity;

			if (processActivity) {
				monitoringModule.maxChatHistoryLength = parseInt((document.getElementById("monitoring_option_chat-history-length") as HTMLInputElement).value || "50");
				monitoringModule.enable();
				if (antiCaptchaEnabled) captchaModule.enable(captchaType, captchaSolveMode);
				radarModule.enable();
			} else {
				monitoringModule.disable();
				captchaModule.disable();
				radarModule.disable();
			}

			updateProcessStatus(statusId, connectedBots, totalBots);
		} catch (error) {
			logger.log(`Ошибка обработки события "process:synchronize": ${error}`, "error");
		}
	});
}

/** Функция инициализации Discord RPC */
async function initDiscordRpc(): Promise<void> {
	const selectDiscordRpcMode = document.getElementById("interface_select_discord-rpc") as HTMLSelectElement;

	selectDiscordRpcMode.addEventListener("change", async () => {
		const result = await invoke<CommandResult<null>>("set_discord_rpc", { state: selectDiscordRpcMode.value === "enable" });
		if (result.error) logger.log(`Ошибка изменения состояния Discord RPC: ${result.error}`, "error");
	});

	const result = await invoke<CommandResult<null>>("set_discord_rpc", { state: selectDiscordRpcMode.value === "enable" });
	if (result.error) logger.log(`Ошибка изменения состояния Discord RPC: ${result.error}`, "error");
}

document.addEventListener("DOMContentLoaded", () => {
	initTitleBar();

	document.addEventListener("contextmenu", e => e.preventDefault());

	logger.init();
	messages.init();

	logger.log(`Клиент запущен, версия ${CLIENT_VERSION}`, "info");

	try {
		timerModule.init();

		initGlobalElements();
		replenishTriggerRegistry();

		document.querySelectorAll('[global="true"]').forEach(c => globalWrappers.push({ id: c.id, el: c as HTMLDivElement }));
		document.querySelectorAll('[sector="true"]').forEach(c => controlWrappers.push({ id: c.id, el: c as HTMLDivElement }));

		enableParticles();

		chartModule.init();
		radarModule.init();
		scriptModule.init();
		pingModule.init();
		stylekitModule.init();
	} catch (error) {
		logger.log(`Ошибка инициализации (sync): ${error}`, "error");
	}
});

document.addEventListener("DOMContentLoaded", async () => {
	try {
		await listenSystemEvents();
		await configModule.init();

		scriptModule.updateLineCounter();

		await accountModule.init();
		await monitoringModule.init();
		await captchaModule.init();
		await proxyModule.init();
		await scannerModule.init();

		await sessionModule.init();

		await translatorModule.init();
		await pluginModule.init();
		await docsModule.init();
		await infoModule.init();

		await initDiscordRpc();

		updaterModule.init();
	} catch (error) {
		logger.log(`Ошибка инициализации (async): ${error}`, "error");
	}
});
