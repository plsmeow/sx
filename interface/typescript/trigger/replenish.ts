import { disableParticles, enableParticles } from "../effects/particles";
import { logger } from "../utils/logger";
import { triggerRegistry } from "./registry";
import { messages } from "../utils/message";

/** Обёрточная функция для пополнения реестра триггеров */
export function replenishTriggerRegistry(): void {
	triggerRegistry.register("settings_select_nickname-type", "select", (current: HTMLSelectElement) => {
		const container = document.getElementById("custom-nickname-template-container") as HTMLInputElement;

		if (current.value === "custom") {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_select_password-type", "select", (current: HTMLSelectElement) => {
		const container = document.getElementById("custom-password-template-container") as HTMLInputElement;

		if (current.value === "custom") {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_select_register-mode", "select", (current: HTMLSelectElement) => {
		if (current.value === "default") {
			(document.getElementById("register-min-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("register-max-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("register-trigger-container") as HTMLInputElement).style.display = "none";
		} else {
			(document.getElementById("register-min-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("register-max-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("register-trigger-container") as HTMLInputElement).style.display = "flex";
		}
	});

	triggerRegistry.register("settings_select_login-mode", "select", (current: HTMLSelectElement) => {
		if (current.value === "default") {
			(document.getElementById("login-min-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("login-max-delay-container") as HTMLInputElement).style.display = "flex";
			(document.getElementById("login-trigger-container") as HTMLInputElement).style.display = "none";
		} else {
			(document.getElementById("login-min-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("login-max-delay-container") as HTMLInputElement).style.display = "none";
			(document.getElementById("login-trigger-container") as HTMLInputElement).style.display = "flex";
		}
	});

	triggerRegistry.register("settings_chbx_use-auto-register", "checkbox", (current: HTMLInputElement) => {
		const container = document.getElementById("auto-register-input-container") as HTMLElement;

		if (current.checked) {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_chbx_use-auto-login", "checkbox", (current: HTMLInputElement) => {
		const container = document.getElementById("auto-login-input-container") as HTMLElement;

		if (current.checked) {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_chbx_use-auto-rejoin", "checkbox", (current: HTMLInputElement) => {
		const container = document.getElementById("auto-rejoin-input-container") as HTMLElement;

		if (current.checked) {
			container.style.display = "flex";
		} else {
			container.style.display = "none";
		}
	});

	triggerRegistry.register("settings_chbx_use-accounts", "checkbox", (current: HTMLInputElement) => {
		const openAccountsSectionBtn = document.getElementById("accounts") as HTMLButtonElement;
		const openProxySectionBtn = document.getElementById("proxy") as HTMLButtonElement;

		const useProxyChbx = document.getElementById("settings_chbx_use-proxy") as HTMLInputElement;
		const botsCountInput = document.getElementById("settings-option-bots-count-container") as HTMLDivElement;
		const selectNicknameType = document.getElementById("settings_select_nickname-type") as HTMLSelectElement;
		const selectPasswordType = document.getElementById("settings_select_password-type") as HTMLSelectElement;
		const selectEmailType = document.getElementById("settings_select_email-type") as HTMLSelectElement;
		const nicknameTemplateInput = document.getElementById("settings_option_nickname-template") as HTMLInputElement;
		const passwordTemplateInput = document.getElementById("settings_option_password-template") as HTMLInputElement;

		if (current.checked) {
			openProxySectionBtn.style.display = "none";
			openAccountsSectionBtn.style.display = "flex";

			useProxyChbx.checked = false;
			useProxyChbx.disabled = true;
			selectNicknameType.disabled = true;
			selectPasswordType.disabled = true;
			selectEmailType.disabled = true;
			nicknameTemplateInput.disabled = true;
			passwordTemplateInput.disabled = true;
			botsCountInput.style.display = "none";
		} else {
			openProxySectionBtn.style.display = "flex";
			openAccountsSectionBtn.style.display = "none";

			useProxyChbx.disabled = false;
			selectNicknameType.disabled = false;
			selectPasswordType.disabled = false;
			selectEmailType.disabled = false;
			nicknameTemplateInput.disabled = false;
			passwordTemplateInput.disabled = false;
			botsCountInput.style.display = "flex";
		}
	});

	triggerRegistry.register("captcha-bypass_select_captcha-type", "select", (current: HTMLSelectElement) => {
		const antiWebCaptchaOptionsContainer = document.getElementById("anti-web-captcha-options") as HTMLElement;
		const antiMapCaptchaOptionsContainer = document.getElementById("anti-map-captcha-options") as HTMLElement;
		const antiMapCaptchaSelectsContainer = document.getElementById("anti-map-captha-selects-container") as HTMLElement;
		const antiMapCaptchaSelectSubtype = document.getElementById("captcha-bypass_select_captcha-subtype") as HTMLSelectElement;
		const antiMapCaptchaNumberOfColumnsInput = document.getElementById("anti-map-captcha-number-of-columns-container") as HTMLElement;
		const antiMapCaptchaNumberOfRowsInput = document.getElementById("anti-map-captcha-number-of-rows-container") as HTMLElement;
		const antiMapCaptchaSelectSizeContainer = document.getElementById("anti-map-captcha-select-size-container") as HTMLElement;
		const antiMapCaptchaMaxPauseInput = document.getElementById("anti-map-captcha-max-pause") as HTMLElement;

		if (current.value === "web") {
			antiWebCaptchaOptionsContainer.style.display = "flex";
			antiMapCaptchaOptionsContainer.style.display = "none";
			antiMapCaptchaSelectsContainer.style.display = "none";
		} else if (current.value === "map") {
			antiWebCaptchaOptionsContainer.style.display = "none";
			antiMapCaptchaSelectsContainer.style.display = "flex";
			antiMapCaptchaOptionsContainer.style.display = "flex";

			if (antiMapCaptchaSelectSubtype.value === "frame") {
				antiMapCaptchaSelectSizeContainer.style.display = "grid";

				const antiMapCaptchaSelectSize = document.getElementById("captcha-bypass_select_captcha-size") as HTMLSelectElement;

				if (antiMapCaptchaSelectSize.value === "dynamic") {
					antiMapCaptchaNumberOfColumnsInput.style.display = "none";
					antiMapCaptchaNumberOfRowsInput.style.display = "none";
					antiMapCaptchaMaxPauseInput.style.display = "flex";
				} else {
					antiMapCaptchaMaxPauseInput.style.display = "none";
					antiMapCaptchaNumberOfColumnsInput.style.display = "flex";
					antiMapCaptchaNumberOfRowsInput.style.display = "flex";
				}
			} else {
				antiMapCaptchaSelectSizeContainer.style.display = "none";
				antiMapCaptchaNumberOfColumnsInput.style.display = "none";
				antiMapCaptchaNumberOfRowsInput.style.display = "none";
				antiMapCaptchaMaxPauseInput.style.display = "none";
			}
		}
	});

	triggerRegistry.register("captcha-bypass_select_solve-mode", "select", (current: HTMLSelectElement) => {
		const antiCaptchaType = (document.getElementById("captcha-bypass_select_captcha-type") as HTMLSelectElement).value;
		const antiMapCaptchaApiInputContainer = document.getElementById("anti-map-captcha-api-key-container") as HTMLElement;
		const antiMapCaptchaApiServiceSelectContainer = document.getElementById("anti-map-captcha-select-api-service-container") as HTMLElement;

		if (current.value === "auto") {
			if (antiCaptchaType === "map") {
				antiMapCaptchaApiInputContainer.style.display = "flex";
				antiMapCaptchaApiServiceSelectContainer.style.display = "grid";
			}
		} else {
			antiMapCaptchaApiInputContainer.style.display = "none";
			antiMapCaptchaApiServiceSelectContainer.style.display = "none";
		}
	});

	triggerRegistry.register("captcha-bypass_select_captcha-subtype", "select", (current: HTMLSelectElement) => {
		const antiMapCaptchaNumberOfColumnsInput = document.getElementById("anti-map-captcha-number-of-columns-container") as HTMLElement;
		const antiMapCaptchaNumberOfRowsInput = document.getElementById("anti-map-captcha-number-of-rows-container") as HTMLElement;
		const antiMapCaptchaSelectSizeContainer = document.getElementById("anti-map-captcha-select-size-container") as HTMLElement;
		const antiMapCaptchaMaxPauseInput = document.getElementById("anti-map-captcha-max-pause") as HTMLElement;

		if (current.value === "inventory") {
			antiMapCaptchaNumberOfColumnsInput.style.display = "none";
			antiMapCaptchaNumberOfRowsInput.style.display = "none";
			antiMapCaptchaSelectSizeContainer.style.display = "none";
			antiMapCaptchaMaxPauseInput.style.display = "none";
		} else if (current.value === "frame") {
			antiMapCaptchaSelectSizeContainer.style.display = "grid";

			const antiMapCaptchaSelectSize = document.getElementById("captcha-bypass_select_captcha-size") as HTMLSelectElement;

			if (antiMapCaptchaSelectSize.value === "dynamic") {
				antiMapCaptchaNumberOfColumnsInput.style.display = "none";
				antiMapCaptchaNumberOfRowsInput.style.display = "none";
				antiMapCaptchaMaxPauseInput.style.display = "flex";
			} else {
				antiMapCaptchaMaxPauseInput.style.display = "none";
				antiMapCaptchaNumberOfColumnsInput.style.display = "flex";
				antiMapCaptchaNumberOfRowsInput.style.display = "flex";
			}
		}
	});

	triggerRegistry.register("captcha-bypass_select_captcha-service", "select", (current: HTMLSelectElement) => {
		const antiMapCaptchaUserIdInput = document.getElementById("anti-map-captcha-user-id-container") as HTMLElement;
		antiMapCaptchaUserIdInput.style.display = current.value === "truecaptcha" ? "flex" : "none";
	});

	triggerRegistry.register("captcha-bypass_select_captcha-size", "select", (current: HTMLSelectElement) => {
		const antiMapCaptchaNumberOfColumnsInput = document.getElementById("anti-map-captcha-number-of-columns-container") as HTMLElement;
		const antiMapCaptchaNumberOfRowsInput = document.getElementById("anti-map-captcha-number-of-rows-container") as HTMLElement;
		const antiMapCaptchaMaxPauseInput = document.getElementById("anti-map-captcha-max-pause") as HTMLElement;

		if (current.value === "dynamic") {
			antiMapCaptchaNumberOfColumnsInput.style.display = "none";
			antiMapCaptchaNumberOfRowsInput.style.display = "none";
			antiMapCaptchaMaxPauseInput.style.display = "flex";
		} else {
			antiMapCaptchaMaxPauseInput.style.display = "none";
			antiMapCaptchaNumberOfColumnsInput.style.display = "flex";
			antiMapCaptchaNumberOfRowsInput.style.display = "flex";
		}
	});

	triggerRegistry.register("module_chat_select_mode", "select", (current: HTMLSelectElement) => {
		const chatSpammingChbxContainer = document.getElementById("chat-spamming-chbx-container") as HTMLElement;
		const chatSpammingInputContainer = document.getElementById("chat-spamming-input-container") as HTMLElement;

		const chatDefaultBtnsContainer = document.getElementById("chat-default-btns-container") as HTMLElement;
		const chatSpammingBtnsContainer = document.getElementById("chat-spamming-btns-container") as HTMLElement;

		if (current.value === "spamming") {
			chatSpammingChbxContainer.style.display = "flex";
			chatSpammingInputContainer.style.display = "flex";
			chatSpammingBtnsContainer.style.display = "flex";

			chatDefaultBtnsContainer.style.display = "none";
		} else {
			chatDefaultBtnsContainer.style.display = "flex";

			chatSpammingChbxContainer.style.display = "none";
			chatSpammingInputContainer.style.display = "none";
			chatSpammingBtnsContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_inventory_select_mode", "select", (current: HTMLSelectElement) => {
		const basicInputContainer = document.getElementById("inventory-basic-input-container") as HTMLElement;
		const basicBtnContainer = document.getElementById("inventory-basic-btn-container") as HTMLElement;
		const swapInputContainer = document.getElementById("inventory-swap-input-container") as HTMLElement;
		const swapBtnContainer = document.getElementById("inventory-swap-btn-container") as HTMLElement;
		const interactionBtnContainer = document.getElementById("inventory-interaction-btn-container") as HTMLElement;

		if (current.value === "basic") {
			basicBtnContainer.style.display = "flex";
			basicInputContainer.style.display = "flex";
			swapInputContainer.style.display = "none";
			swapBtnContainer.style.display = "none";
			interactionBtnContainer.style.display = "none";
		} else if (current.value === "swap") {
			swapInputContainer.style.display = "flex";
			swapBtnContainer.style.display = "flex";
			basicInputContainer.style.display = "flex";
			basicBtnContainer.style.display = "none";
			interactionBtnContainer.style.display = "none";
		} else if (current.value === "interaction") {
			swapInputContainer.style.display = "none";
			swapBtnContainer.style.display = "none";
			basicInputContainer.style.display = "none";
			basicBtnContainer.style.display = "none";
			interactionBtnContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("module_movement_select_mode", "select", (current: HTMLSelectElement) => {
		const selectMovementDirectionContainer = document.getElementById("select-movement-direction-container") as HTMLElement;
		const movementPathfinderInputContainer = document.getElementById("movement-pathfinder-input-container") as HTMLElement;
		const movementImpulsivenessContainer = document.getElementById("movement-impulsiveness-container") as HTMLElement;

		if (current.value === "default") {
			selectMovementDirectionContainer.style.display = "flex";
			movementPathfinderInputContainer.style.display = "none";
			movementImpulsivenessContainer.style.display = "flex";
		} else {
			selectMovementDirectionContainer.style.display = "none";
			movementPathfinderInputContainer.style.display = "flex";
			movementImpulsivenessContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_flight_select_settings", "select", (current: HTMLSelectElement) => {
		const manualSettingsContainer = document.getElementById("flight-manual-settings-container") as HTMLElement;
		const selectAntiCheatContainer = document.getElementById("flight-select-anti-cheat-container") as HTMLElement;

		if (current.value === "adaptive") {
			manualSettingsContainer.style.display = "none";
			selectAntiCheatContainer.style.display = "grid";
		} else {
			manualSettingsContainer.style.display = "flex";
			selectAntiCheatContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_killaura_select_target", "select", (current: HTMLSelectElement) => {
		const customGoalInputContainer = document.getElementById("killaura-custom-goal-input-container") as HTMLElement;

		if (current.value === "custom") {
			customGoalInputContainer.style.display = "flex";
		} else {
			customGoalInputContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_killaura_select_settings", "select", (current: HTMLSelectElement) => {
		const manualSettingsContainer = document.getElementById("killaura-manual-settings-container") as HTMLElement;

		if (current.value === "adaptive") {
			manualSettingsContainer.style.display = "none";
		} else {
			manualSettingsContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("module_killaura_chbx_use-auto-weapon", "checkbox", (current: HTMLInputElement) => {
		const selectWeaponContainer = document.getElementById("select-killaura-weapon-container") as HTMLElement;
		const weaponSlotContainer = document.getElementById("killaura-weapon-slot-container") as HTMLElement;

		if (current.checked) {
			selectWeaponContainer.style.display = "grid";
			weaponSlotContainer.style.display = "none";
		} else {
			selectWeaponContainer.style.display = "none";
			weaponSlotContainer.style.display = "flex";
		}
	});

	triggerRegistry.register("module_killaura_chbx_use-chase", "checkbox", (current: HTMLInputElement) => {
		const chaseSettingsContainer = document.getElementById("killaura-chase-settings-container") as HTMLElement;

		if (current.checked) {
			chaseSettingsContainer.style.display = "flex";
		} else {
			chaseSettingsContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_bow-aim_select_target", "select", (current: HTMLSelectElement) => {
		const customGoalInputContainer = document.getElementById("bow-aim-custom-goal-input-container") as HTMLElement;

		if (current.value === "custom") {
			customGoalInputContainer.style.display = "flex";
		} else {
			customGoalInputContainer.style.display = "none";
		}
	});

	triggerRegistry.register("module_miner_select_mode", "select", (current: HTMLSelectElement) => {
		const tunnelSelectContainer = document.getElementById("miner-tunnel-select-container") as HTMLElement;
		const lookSelectContainer = document.getElementById("miner-look-select-container") as HTMLElement;

		if (current.value === "default") {
			tunnelSelectContainer.style.display = "none";
			lookSelectContainer.style.display = "none";
		} else {
			tunnelSelectContainer.style.display = "grid";
			lookSelectContainer.style.display = "grid";
		}
	});

	triggerRegistry.register("journal_chbx_show-system-logs", "checkbox", (current: HTMLInputElement) => logger.setVisibility("system", current.checked));
	triggerRegistry.register("journal_chbx_show-extended-logs", "checkbox", (current: HTMLInputElement) => logger.setVisibility("extended", current.checked));

	triggerRegistry.register("proxy-finder_select_algorithm", "select", (current: HTMLSelectElement) => {
		const selectProxyFinderCountry = document.getElementById("proxy-finder_select_country") as HTMLSelectElement;

		if (current.value === "trusted") {
			selectProxyFinderCountry.disabled = false;
		} else {
			selectProxyFinderCountry.disabled = true;
		}
	});

	triggerRegistry.register("interface_select_particles", "select", (current: HTMLSelectElement) => {
		try {
			if (current.value === "enable") {
				enableParticles();
			} else {
				disableParticles();
			}
		} catch (error) {
			logger.log(`Ошибка изменения состояния партиклов: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_global-font-family", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;

			switch (current.value) {
				case "inter":
					root.style.setProperty("--global-font-family", `"Inter", sans-serif`);
					break;
				case "jetbrains-mono":
					root.style.setProperty("--global-font-family", `"JetBrains Mono", "Fira Code", monospace`);
					break;
				case "segoe-ui":
					root.style.setProperty("--global-font-family", `"Segoe UI", Tahoma, Geneva, Verdana, sans-serif`);
					break;
			}
		} catch (error) {
			logger.log(`Ошибка изменения шрифта: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_show-messages", "select", (current: HTMLSelectElement) => {
		try {
			messages.visibility(current.value);

			document.querySelectorAll<HTMLElement>(".message").forEach(m => {
				if (current.value === "hide") {
					m.style.display = "none";
				} else {
					m.style.display = "flex";
				}
			});
		} catch (error) {
			logger.log(`Ошибка изменения состояния сообщений: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_show-panel-icons", "select", (current: HTMLSelectElement) => {
		try {
			const display = current.value === "hide" ? "none" : "flex";
			document.querySelectorAll<SVGElement>(".dashboard button svg").forEach(i => i.style.display = display);
		} catch (error) {
			logger.log(`Ошибка изменения состояния иконок на панели: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-font-family", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;

			switch (current.value) {
				case "inter":
					root.style.setProperty("--panel-font-family", `"Inter", sans-serif`);
					root.style.setProperty("--panel-text-margin-top", "1px");
					break;
				case "jetbrains-mono":
					root.style.setProperty("--panel-font-family", `"JetBrains Mono", "Fira Code", monospace`);
					root.style.setProperty("--panel-text-margin-top", "2px");
					break;
				case "segoe-ui":
					root.style.setProperty("--panel-font-family", `"Segoe UI", Tahoma, Geneva, Verdana, sans-serif`);
					root.style.setProperty("--panel-text-margin-top", "-1px");
					break;
				case "courier-new":
					root.style.setProperty("--panel-font-family", `"Courier New", Courier, monospace`);
					root.style.setProperty("--panel-text-margin-top", "4px");
					break;
				case "trebuchet-ms":
					root.style.setProperty("--panel-font-family", `"Trebuchet MS", "Lucida Sans Unicode", "Lucida Grande", "Lucida Sans", Arial, sans-serif`);
					root.style.setProperty("--panel-text-margin-top", "1px");
					break;
			}
		} catch (error) {
			logger.log(`Ошибка изменения шрифта панели: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-font-size", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;
			root.style.setProperty("--panel-font-size", current.value);
		} catch (error) {
			logger.log(`Ошибка изменения размера шрифта панели: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-internal-gap", "select", (current: HTMLSelectElement) => {
		try {
			const root = document.documentElement;
			root.style.setProperty("--panel-btn-gap", current.value);
		} catch (error) {
			logger.log(`Ошибка изменения внутреннего отступа кнопок панели: ${error}`, "error");
		}
	});

	triggerRegistry.register("interface_select_panel-separators", "select", (current: HTMLSelectElement) => {
		try {
			const display = current.value === "hide" ? "none" : "flex";
			document.querySelectorAll<HTMLElement>(".dashboard .splitter").forEach(s => s.style.display = display);
		} catch (error) {
			logger.log(`Ошибка изменения видимости разделителей панели: ${error}`, "error");
		}
	});
}
