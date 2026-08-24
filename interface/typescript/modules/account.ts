import { open } from "@tauri-apps/plugin-dialog";

import { date } from "../utils/date";
import { logger } from "../utils/logger";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";
import { join } from "@tauri-apps/api/path";
import { messages } from "../utils/message";
import { invoke } from "@tauri-apps/api/core";

/** Структура данных аккаунта */
interface Account {
	creation_date: string;
	initial_group: string | null;
	password: string | null;
	email: string | null;
	proxy: string | null;
	selected: boolean;
}

/** Модуль управления аккаунтами */
class AccountModule {
	private accountList: HTMLDivElement | null = null;
	private accountCounter: HTMLSpanElement | null = null;
	private selectedAccountCounter: HTMLSpanElement | null = null;
	private wrappers: HTMLDivElement | null = null;

	private accounts: Map<string, Account> = new Map();

	private noSelected: boolean = true;

	/** Метод инициализации функций, связанных с аккаунтами */
	public async init(): Promise<void> {
		this.accountList = document.getElementById("account-list") as HTMLDivElement;
		this.accountCounter = document.getElementById("account-counter") as HTMLSpanElement;
		this.selectedAccountCounter = document.getElementById("selected-account-counter") as HTMLSpanElement;
		this.wrappers = document.getElementById("account-wrappers") as HTMLDivElement;

		await this.loadSavedAccounts();

		const changeAccountPanelBtn = document.getElementById("change-account-panel") as HTMLButtonElement;

		const selectedAccountsEditor = document.getElementById("selected-accounts-editor") as HTMLDivElement;
		const selectedAccountsInitialGroup = document.getElementById("selected-accounts-initial-group") as HTMLInputElement;
		const selectedAccountsPassword = document.getElementById("selected-accounts-password") as HTMLInputElement;
		const selectedAccountsEmail = document.getElementById("selected-accounts-email") as HTMLInputElement;
		const selectedAccountsProxy = document.getElementById("selected-accounts-proxy") as HTMLInputElement;

		const addAccountBtn = document.getElementById("add-account") as HTMLButtonElement;
		const openSelectedAccountsEditorBtn = document.getElementById("edit-selected-accounts") as HTMLButtonElement;
		const closeSelectedAccountsEditorBtn = document.getElementById("close-selected-accounts-editor") as HTMLButtonElement;
		const removeSelectedAccountsBtn = document.getElementById("remove-selected-accounts") as HTMLButtonElement;
		const importAccounts = document.getElementById("import-accounts") as HTMLButtonElement;
		const exportAccounts = document.getElementById("export-accounts") as HTMLButtonElement;

		const changeAllAccountsSelect = document.getElementById("change-all-accounts-select") as HTMLInputElement;

		const generateAccountsBtn = document.getElementById("generate-accounts") as HTMLButtonElement;

		changeAccountPanelBtn.addEventListener("click", () => {
			const defaultAccountPanel = document.getElementById("default-account-panel");
			const generationAccountPanel = document.getElementById("generation-account-panel");
			if (!defaultAccountPanel || !generationAccountPanel) return;

			if (generationAccountPanel.style.display === "none") {
				defaultAccountPanel.style.display = "none";
				generationAccountPanel.style.display = "flex";
			} else {
				defaultAccountPanel.style.display = "flex";
				generationAccountPanel.style.display = "none";
			}
		});

		changeAllAccountsSelect.addEventListener("change", () => {
			let count = 0;

			for (const [username, account] of this.accounts) {
				account.selected = changeAllAccountsSelect.checked;
				(document.getElementById(`account-${username}-selector-flag`) as HTMLInputElement).checked = changeAllAccountsSelect.checked;
				count++;
			}

			if (count > 0) {
				this.noSelected = !changeAllAccountsSelect.checked;
				this.changeSelectorsState();
				this.updateSelectedAccountCounter();
			}
		});

		removeSelectedAccountsBtn.addEventListener("click", async () => await this.removeSelectedAccounts());
		openSelectedAccountsEditorBtn.addEventListener("click", () => selectedAccountsEditor.style.display = "flex");
		closeSelectedAccountsEditorBtn.addEventListener("click", () => selectedAccountsEditor.style.display = "none");

		selectedAccountsInitialGroup.addEventListener("input", () => this.setValueForAllSelected("initial_group", selectedAccountsInitialGroup.value));
		selectedAccountsPassword.addEventListener("input", () => this.setValueForAllSelected("password", selectedAccountsPassword.value));
		selectedAccountsEmail.addEventListener("input", () => this.setValueForAllSelected("email", selectedAccountsEmail.value));
		selectedAccountsProxy.addEventListener("input", () => this.setValueForAllSelected("proxy", selectedAccountsProxy.value));

		addAccountBtn.addEventListener("click", () => this.addAccount());
		importAccounts.addEventListener("click", async () => await this.importAccounts());
		exportAccounts.addEventListener("click", async () => await this.exportAccounts());
		generateAccountsBtn.addEventListener("click", () => this.generateAccounts());

		this.watch();
	}

	/** Метод получения всех выбранных аккаунтов без лишних полей */
	public getSelectedAccounts(): Map<string, any> {
		let cleanAccounts: Map<string, any> = new Map();

		for (const [username, account] of this.accounts) {
			if (!account.selected) continue;

			cleanAccounts.set(username, {
				initial_group: account.initial_group,
				password: account.password,
				email: account.email,
				proxy: account.proxy,
			});
		}

		return cleanAccounts;
	}

	/** Метод запуска цикла сохранения пользовательских аккаунтов */
	private watch(): void {
		let latestSize = 0;

		setInterval(async () => {
			// Нету смысла сохранять пустой JSON, поэтому такая проверка
			if (this.accounts.size < 1 && latestSize > 0) {
				return;
			}

			let cleanAccounts: any = {};

			for (const [username, account] of this.accounts) {
				cleanAccounts[username] = {
					creation_date: account.creation_date,
					initial_group: account.initial_group,
					password: account.password,
					email: account.email,
					proxy: account.proxy,
				};
			}

			const string = JSON.stringify(cleanAccounts, null, 2);
			const encoder = new TextEncoder();
			const encoded = encoder.encode(string);

			const result = await invoke<CommandResult<null>>("save_file", {
				path: "accounts.json",
				content: encoded,
			});

			if (result.error) {
				logger.log(`Ошибка сохранения аккаунтов: ${result.error}`, "error");
				return;
			}

			latestSize = this.accounts.size;
		}, 2000);
	}

	/** Метод обновления счётчика аккаунтов */
	private updateAccountCounter(): void {
		if (!this.accountCounter) return;
		this.accountCounter.innerText = this.accounts.size.toString();
	}

	/** Метод обновления счётчика выбранных аккаунтов */
	private updateSelectedAccountCounter(): void {
		if (!this.selectedAccountCounter) return;

		let count = 0;
		for (const [_, account] of this.accounts) account.selected ? count++ : null;

		if (count < 1) {
			this.selectedAccountCounter.innerText = "";
		} else {
			this.selectedAccountCounter.innerText = `(${count})`;
		}
	}

	/** Метод измнения состояния кнопок, отвечающих за взаимодействие с выбранными аккаунтами */
	private changeSelectorsState(): void {
		(document.getElementById("edit-selected-accounts") as HTMLButtonElement).disabled = this.noSelected;
		(document.getElementById("remove-selected-accounts") as HTMLButtonElement).disabled = this.noSelected;
	}

	/** Метод измнения значения всех выбранных аккаунтов */
	private setValueForAllSelected(
		field: "initial_group" | "password" | "email" | "proxy",
		value: string,
	): void {
		for (const [username, account] of this.accounts) {
			if (!account.selected) continue;

			account[field] = value;

			const input = document.getElementById(`account-${username}-${field}`) as HTMLInputElement | null;
			if (input) input.value = value;
		}
	}

	/** Метод полной очистки всех аккаунтов */
	private async clearAccounts(): Promise<void> {
		document.querySelectorAll('[wrapper="account-editor"]').forEach(w => w.remove());
		document.querySelectorAll('[wrapper="account-card"]').forEach(w => w.remove());

		this.accounts.clear();

		const result = await invoke<CommandResult<null>>("remove_file", { path: "accounts.json" });
		if (result.error) logger.log(`Ошибка удаления файла "accounts.json": ${result.error}`, "error");

		this.noSelected = true;
		this.changeSelectorsState();
		this.updateAccountCounter();
		this.updateSelectedAccountCounter();
	}

	/** Метод загрузки сохранённых аккаунтов из локального хранилища */
	private async loadSavedAccounts(): Promise<void> {
		const result = await invoke<CommandResult<number[]>>("read_file", { path: "accounts.json" });

		if (result.error) {
			logger.log(`Ошибка загрузки контента из файла "accounts.json": ${result.error}`, "error");
			return;
		}

		if (!result.data || result.data.length < 1) return;

		const uint8arr = new Uint8Array(result.data);
		const decoder = new TextDecoder("utf-8");
		const decoded = decoder.decode(uint8arr);
		const accounts = Object.entries<Account>(JSON.parse(decoded));

		if (accounts.length < 1) return;

		for (const [username, account] of accounts) this.createAccountCard(username, account);

		this.updateAccountCounter();

		logger.log(`Сохранённые аккаунты загружены (${accounts.length > 255 ? 255 : accounts.length} штук)`, "system");
	}

	/** Метод добавления аккаунта */
	private addAccount(): void {
		const usernameInput = document.getElementById("account-username") as HTMLInputElement;
		const username = usernameInput.value;

		const status = this.createAccountCard(username, {
			creation_date: date("exact"),
			initial_group: null,
			password: null,
			email: null,
			proxy: null,
			selected: false,
		});

		switch (status) {
			case 0:
				this.updateAccountCounter();
				usernameInput.value = "";
				logger.log(`Создан новый аккаунт "${username}"`, "system");
				break;
			case 1:
				logger.log(`Не удалось создать аккаунт "${username}": Accounts with empty usernames are prohibited`, "warning");
				break;
			case 2:
				logger.log(`Не удалось создать аккаунт "${username}": Account with this username already exists`, "warning");
				break;
			case 3:
				logger.log(`Не удалось создать аккаунт "${username}": Number of generated accounts is out of limits (minimum 1, maximum 200)`, "warning");
				break;
		}

		if (status !== 0) {
			messages.message("Аккаунты", `Не удалось создать аккаунт "${username}"`);
		}
	}

	/** Метод удаления всех выбранных аккаунтов */
	private async removeSelectedAccounts(): Promise<void> {
		let selectedCount = 0;
		for (const [_, account] of this.accounts) account.selected ? selectedCount++ : null;

		if (selectedCount === this.accounts.size) {
			await this.clearAccounts();
			return;
		}

		let removedCount = 0;
		for (const [username, account] of this.accounts) {
			if (!account.selected) continue;

			this.accounts.delete(username);

			document.getElementById(`account-editor-${username}`)?.remove();
			document.getElementById(`account-card-${username}`)?.remove();

			removedCount++;
		}

		this.noSelected = true;
		this.changeSelectorsState();
		this.updateAccountCounter();
		this.updateSelectedAccountCounter();

		messages.message("Аккаунты", `Удалено ${removedCount} аккаунтов`);
		logger.log(`Удалено ${removedCount} аккаунтов`, "system");
	}

	/** Метод импортировки аккаунтов */
	private async importAccounts(): Promise<void> {
		try {
			const path = await open({
				directory: false,
				multiple: false,
				filters: [{
					name: "Accounts",
					extensions: ["json"]
				}]
			});

			if (!path) return;

			this.clearAccounts();

			const buffer = await readFile(path);

			const decoder = new TextDecoder();
			const accounts = JSON.parse(decoder.decode(buffer));

			for (const username in accounts) {
				const account = accounts[username];
				if (!account) continue;

				this.createAccountCard(username, {
					creation_date: date("exact"),
					initial_group: account["initial_group"],
					password: account["password"],
					email: account["email"],
					proxy: account["proxy"],
					selected: false,
				});
			}

			this.updateAccountCounter();

			messages.message("Импорт аккаунтов", `Аккаунты успешно импортированы`);
		} catch (error) {
			logger.log(`Ошибка импорта аккаунтов: ${error}`, "error");
		}
	}

	/** Метод экспортировки аккаунтов */
	private async exportAccounts(): Promise<void> {
		try {
			const directory = await open({
				directory: true,
				multiple: false
			});

			if (!directory) return;

			const accounts: any = {};

			for (const [username, account] of this.accounts) {
				accounts[username] = {
					initial_group: account.initial_group,
					password: account.password,
					email: account.email,
					proxy: account.proxy,
				};
			}

			const path = await join(directory, "accounts.json");
			let encoder = new TextEncoder();
			let buffer = encoder.encode(JSON.stringify(accounts, null, 2));

			await writeFile(path, buffer);

			messages.message("Экспорт аккаунтов", `Аккаунты успешно экспортированы`);
		} catch (error) {
			logger.log(`Ошибка экспорта аккаунтов: ${error}`, "error");
		}
	}

	/** Метод генерации аккаунтов по шаблону */
	private generateAccounts(): void {
		const template = (document.getElementById("account-username-template") as HTMLInputElement).value;
		const count = parseInt((document.getElementById("generated-account-count") as HTMLInputElement).value);

		if (template === "") return;

		if (count > 255 || count < 1) {
			messages.message("Аккаунты", "Не удалось сгенерировать аккаунты");
			logger.log(`Не удалось сгенерировать аккаунты: Number of generated accounts is out of limits (minimum 1, maximum 255)`, "warning");
			return;
		}

		let generated = 0;

		for (let i = 0; i < count; i++) {
			let username = "";

			for (let a = 0; a < 5; a++) {
				const generatedUsername = this.generateUsername(i, template);
				if (this.accounts.has(generatedUsername)) continue;
				username = generatedUsername;
				break;
			}

			const status = this.createAccountCard(username, {
				creation_date: date("exact"),
				initial_group: null,
				password: null,
				email: null,
				proxy: null,
				selected: false,
			});

			if (status === 0) generated++;
		}

		this.updateAccountCounter();

		messages.message("Аккаунты", `Сгенерировано ${generated} аккаунтов`);
		logger.log(`Сгенерировано ${generated} аккаунтов по шаблону "${template}"`, "system");
	}

	/** Метод генерации юзернейма */
	private generateUsername(index: number, template: string): string {
		const chars = {
			"numeric": "0123456789",
			"letter": "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
			"multi": "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
		};

		const threeRandom = (type: "numeric" | "letter" | "multi"): string => {
			let result = "";
			for (let i = 0; i < 3; i++) {
				result += chars[type][Math.floor(Math.random() * chars[type].length)];
			}

			return result;
		}

		const username = template
			.replace(/#n/g, () => threeRandom("numeric"))
			.replace(/#l/g, () => threeRandom("letter"))
			.replace(/#m/g, () => threeRandom("multi"))
			.replace(/#i/g, () => index.toString())
			.slice(0, 16);

		return username;
	}

	/** Метод создания карточки аккаунта */
	private createAccountCard(username: string, account: Account): number {
		if (username === "") return 1;
		if (this.accounts.has(username)) return 2;
		if (this.accounts.size >= 255) return 3;

		this.accounts.set(username, account);

		const editorWrapper = this.createEditWrapper(username, account);

		const row = document.createElement("div");
		row.className = "account-row";
		row.id = `account-card-${username}`;
		row.setAttribute("wrapper", "account-card");

		const usernameCell = document.createElement("div");
		usernameCell.className = "account-cell username-cell";

		const chbxContainer = document.createElement("label");
		chbxContainer.className = "checkbox-container";

		const accountSelectorFlag = document.createElement("input");
		accountSelectorFlag.id = `account-${username}-selector-flag`;
		accountSelectorFlag.type = "checkbox";

		accountSelectorFlag.addEventListener("change", () => {
			const account = this.accounts.get(username);
			if (!account) return;

			account.selected = accountSelectorFlag.checked;

			if (accountSelectorFlag.checked) {
				if (this.noSelected) this.noSelected = false;
			} else {
				let noSelected = true;

				for (const [_, a] of this.accounts) {
					if (a.selected) {
						noSelected = false;
						break;
					}
				}

				this.noSelected = noSelected;
			}

			this.changeSelectorsState();
			this.updateSelectedAccountCounter();
		});

		const accountSelectorMark = document.createElement("span");
		accountSelectorMark.className = "checkmark";

		const usernameText = document.createElement("span");
		usernameText.innerText = username;

		chbxContainer.appendChild(accountSelectorFlag);
		chbxContainer.appendChild(accountSelectorMark);
		chbxContainer.appendChild(usernameText);
		usernameCell.appendChild(chbxContainer);

		const creationDateCell = document.createElement("div");
		creationDateCell.className = "account-cell date-cell";
		creationDateCell.innerText = account.creation_date;

		const actionsCell = document.createElement("div");
		actionsCell.className = "account-cell actions-cell";

		const editAccountBtn = document.createElement("button");
		editAccountBtn.className = "action-btn";
		editAccountBtn.addEventListener("click", () => editorWrapper.style.display = "flex");
		editAccountBtn.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-settings">
        <path stroke="none" d="M0 0h24v24H0z" fill="none" />
        <path d="M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.572 1.065c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.065 -2.572c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1 .608 2.296 .07 2.572 -1.065" />
        <path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" />
      </svg>
    `;

		const removeAccountBtn = document.createElement("button");
		removeAccountBtn.className = "action-btn";

		removeAccountBtn.addEventListener("click", () => {
			row.remove();
			editorWrapper.remove();
			this.accounts.delete(username);
			this.updateAccountCounter();
		});

		removeAccountBtn.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-trash">
        <path stroke="none" d="M0 0h24v24H0z" fill="none" />
        <path d="M4 7l16 0" />
        <path d="M10 11l0 6" />
        <path d="M14 11l0 6" />
        <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
        <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
      </svg>
    `;

		actionsCell.appendChild(editAccountBtn);
		actionsCell.appendChild(removeAccountBtn);

		row.appendChild(usernameCell);
		row.appendChild(creationDateCell);
		row.appendChild(actionsCell);

		this.accountList?.appendChild(row);

		return 0;
	}

	/** Метод создания обёркти редактора настроек аккаунта */
	private createEditWrapper(username: string, account: any): HTMLDivElement {
		const wrapper = document.createElement("div");
		wrapper.className = "cover";
		wrapper.id = `account-editor-${username}`;
		wrapper.setAttribute("wrapper", "account-editor");

		const coverPanel = document.createElement("div");
		coverPanel.className = "panel with-header";
		coverPanel.style.marginBottom = "20px";

		const coverPanelLeft = document.createElement("div");
		coverPanelLeft.className = "left";
		coverPanelLeft.innerHTML = `<div class="header">Редактирование аккаунта "${username}"</div>`;

		const coverPanelRight = document.createElement("div");
		coverPanelRight.className = "right";

		const closeEditorBtn = document.createElement("button");
		closeEditorBtn.className = "btn min";
		closeEditorBtn.addEventListener("click", () => wrapper.style.display = "none");
		closeEditorBtn.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
        <path stroke="none" d="M0 0h24v24H0z" fill="none" />
        <path d="M18 6l-12 12" />
        <path d="M6 6l12 12" />
      </svg>
    `;

		coverPanelRight.appendChild(closeEditorBtn);

		coverPanel.appendChild(coverPanelLeft);
		coverPanel.appendChild(coverPanelRight);

		const settingsContainer = document.createElement("div");
		settingsContainer.className = "account-settings";

		const options = [
			{
				name: "Первоначальная группа",
				tag: "initial-group",
				placeholder: "my_group",
			},
			{
				name: "Пароль",
				tag: "password",
				placeholder: "qwerty12345",
			},
			{
				name: "Эл. почта",
				tag: "email",
				placeholder: "qwerty@gmail.com",
			},
			{
				name: "Прокси SOCKS5",
				tag: "proxy",
				placeholder: "socks5://35.91.83.91:1111",
			},
		];

		for (const option of options) {
			const e = document.createElement("div");
			e.className = "pretty-input-wrapper";

			const signature = document.createElement("p");
			signature.className = "signature fix";
			signature.innerText = option.name;

			const input = document.createElement("input");
			input.id = `account-${username}-${option.tag}`;
			input.type = "text";
			input.placeholder = option.placeholder;

			const fieldName = option.tag.replaceAll("-", "_");
			account[fieldName] ? input.value = account[fieldName] : null;
			input.addEventListener("input", () => account[fieldName] = input.value);

			e.appendChild(signature);
			e.appendChild(input);

			settingsContainer.appendChild(e);
		}

		wrapper.appendChild(coverPanel);
		wrapper.appendChild(settingsContainer);

		this.wrappers?.appendChild(wrapper);

		return wrapper;
	}
}

const accountModule = new AccountModule();

export { accountModule }
