import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { setQuickTasksAllowed } from "../main";
import { Buffer } from "../utils/buffer";

/** Модуль управления пользовательским сценарием */
class ScriptModule {
	private editor: HTMLTextAreaElement | null = null;
	private lineCounter: HTMLDivElement | null = null;

	/** Метод инициализации функций, связанных с скриптингом */
	public init(): void {
		this.editor = document.getElementById("user-script") as HTMLTextAreaElement;
		this.lineCounter = document.getElementById("line-counter") as HTMLDivElement;

		if (!this.editor) return;

		this.editor.addEventListener("keydown", (e) => {
			if (e.key === "Tab" && this.editor) {
				e.preventDefault();
				const start = this.editor.selectionStart;
				const end = this.editor.selectionEnd;
				const value = this.editor.value;
				this.editor.value = value.substring(0, start) + "  " + value.substring(end);
				this.editor.selectionStart = this.editor.selectionEnd = start + 2;
				this.updateLineCounter();
			}
		});

		this.editor.addEventListener("mouseenter", () => setQuickTasksAllowed(false));
		this.editor.addEventListener("mouseleave", () => setQuickTasksAllowed(true));
		this.editor.addEventListener("input", () => this.updateLineCounter());
		this.editor.addEventListener("scroll", () => this.lineCounter && this.editor ? this.lineCounter.scrollTop = this.editor.scrollTop : null);

		document.getElementById("execute-script")?.addEventListener("click", async () => await this.execute(false));
		document.getElementById("execute-script-separately")?.addEventListener("click", async () => await this.execute(true));
		document.getElementById("stop-script")?.addEventListener("click", async () => await this.stop());
	}

	/** Метод обновления счётчика строк */
	public updateLineCounter(): void {
		if (!this.editor || !this.lineCounter) return;
		const lines = this.editor.value.split("\n").length;
		let numbers = "";
		for (let i = 1; i <= lines; i++) numbers += `<p>${i}</p>\n`;
		this.lineCounter.innerHTML = numbers;
	}

	/** Метод исполнения пользовательского сценария */
	private async execute(separately: boolean): Promise<void> {
		try {
			if (!this.editor) return;
			const script = this.editor.value;
			if (script === "") return;

			const buf = new Buffer();
			buf.writeU8(0x0C);
			buf.writeString(script);
			buf.writeBoolean(separately);

			const result = await invoke<CommandResult<null>>("send_command", {
				data: buf.toUint8Array(),
			});

			if (result.error) logger.log(`Ошибка выполнения сценария: ${result.error}`, "error");
		} catch (error) {
			logger.log(`Ошибка выполнения сценария: ${error}`, "error");
		}
	}

	/** Метод остановки пользовательского сценария */
	private async stop(): Promise<void> {
		try {
			const buf = new Buffer();
			buf.writeU8(0x0D);
			buf.writeU32(0);

			const result = await invoke<CommandResult<null>>("send_command", {
				data: buf.toUint8Array(),
			});

			if (result.error) logger.log(`Ошибка остановки сценария: ${result.error}`, "error");
		} catch (error) {
			logger.log(`Ошибка остановки сценария: ${error}`, "error");
		}
	}
}

const scriptModule = new ScriptModule();

export { scriptModule }
