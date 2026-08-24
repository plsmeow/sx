import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { messages } from "../utils/message";
import { logger } from "../utils/logger";
import { Buffer } from "../utils/buffer";

interface ServerMeta {
	address: string;
	icon: string | null;
	onlinePlayers: number;
	maxPlayers: number;
	version: string;
}

/** Модуль сканирования хостов на наличие Minecraft серверов */
class ScannerModule {
	private serverList: HTMLDivElement | null = null;

	/** Метод инициализации модуля */
	public async init(): Promise<void> {
		this.serverList = document.getElementById("scanner-server-list") as HTMLDivElement;

		const ipRangeInput = document.getElementById("scanner-ip-range") as HTMLInputElement;
		const taskCountInput = document.getElementById("scanner-task-count") as HTMLInputElement;
		const targetPortInput = document.getElementById("scanner-target-port") as HTMLInputElement;
		const timeoutInput = document.getElementById("scanner-timeout") as HTMLInputElement;

		document.getElementById("start-network-scanning")?.addEventListener("click", async () => {
			const range = ipRangeInput.value;
			if (range === "") return;

			if (this.serverList) this.serverList.innerHTML = "";

			const taskCount = parseInt(taskCountInput.value);
			const targetPort = parseInt(targetPortInput.value === "" ? "25565" : targetPortInput.value);
			const timeout = parseInt(timeoutInput.value);

			const result = await invoke<CommandResult<null>>("start_network_scanning", {
				range: range,
				taskCount: taskCount < 1 ? 10 : taskCount,
				targetPort: targetPort,
				timeout: timeout < 1 ? 8000 : timeout,
			});

			if (result.error) {
				messages.message("Сканер", `Ошибка сканирования ${range}`);
				logger.log(`Ошибка сканирования серверов в диапазоне ${range}: ${result.error}`, "error");
			} else {
				messages.message("Сканер", `Сканирование ${range}...`);
				logger.log(`Запущено сканирование серверов в диапазоне ${range}`, "system");
			}
		});

		document.getElementById("stop-network-scanning")?.addEventListener("click", async () => {
			await invoke("stop_network_scanning");
			messages.message("Сканер", "Сканирование серверов остановлено");
			logger.log("Сканирование серверов остановлено", "system");
		});

		await listen<number[]>("scanner:push-server", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));

				this.pushServerToList({
					address: buf.readString(),
					icon: buf.readOption(() => buf.readString()),
					onlinePlayers: buf.readI32(),
					maxPlayers: buf.readI32(),
					version: buf.readString(),
				});
			} catch (error) {
				logger.log(`Ошибка обработки события "scanner:push-server": ${error}`, "error");
			}
		});

		const scannerStatus = document.getElementById("scanner-status") as HTMLSpanElement;
		await listen<number[]>("scanner:status", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				scannerStatus.innerText = buf.readU8() === 0x00 ? "inactive" : "active";
			} catch (error) {
				logger.log(`Ошибка обработки события "scanner:status": ${error}`, "error");
			}
		});

		const hostsScanned = document.getElementById("scanner-hosts-scanned") as HTMLSpanElement;
		await listen<number[]>("scanner:hosts-scanned", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const scanned = buf.readU16();
				const total = buf.readU16();
				hostsScanned.innerText = `${scanned}/${total}`;
			} catch (error) {
				logger.log(`Ошибка обработки события "scanner:hosts-scanned": ${error}`, "error");
			}
		});

		const serversFound = document.getElementById("scanner-servers-found") as HTMLSpanElement;
		await listen<number[]>("scanner:servers-found", (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				serversFound.innerText = buf.readU16().toString();
			} catch (error) {
				logger.log(`Ошибка обработки события "scanner:servers-found": ${error}`, "error");
			}
		});
	}

	/** Метод добавления сервера в список */
	private pushServerToList(meta: ServerMeta): void {
		if (!this.serverList) return;

		const card = document.createElement("div");
		card.className = "server";
		card.innerHTML = `
			${meta.icon ? `<img src=\"${meta.icon}\" draggable="false">` : ""}
			<div class="info">
				<label class="header">
          <span class="ip">${meta.address}</span>
          <span class="players">(${meta.onlinePlayers}/${meta.maxPlayers})</span>
        </label>
				<label class="version">${meta.version}</label>
			</div>
		`;

		this.serverList.appendChild(card);
	}
}

const scannerModule = new ScannerModule();

export { scannerModule }
