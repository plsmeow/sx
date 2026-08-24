import { invoke } from "@tauri-apps/api/core";
import { isAbsolute } from "@tauri-apps/api/path";
import { Chart } from "chart.js";

import { logger } from "../utils/logger";
import { messages } from "../utils/message";
import { Buffer } from "../utils/buffer";
import { listen } from "@tauri-apps/api/event";

/** Модуль обнаружения игроков */
class RadarModule {
	private active: boolean = false;

	private targetCardsContainer: HTMLElement | null = null;
	private targetWrappersContainer: HTMLElement | null = null;

	private updateFrequency: number = 1500;
	private targets: Map<string, { data: any, interval: any, chart: any }> = new Map();

	/** Метод инициализации функций, связанных с радаром */
	public async init(): Promise<void> {
		this.targetCardsContainer = document.getElementById("radar-target-cards-container") as HTMLElement;
		this.targetWrappersContainer = document.getElementById("radar-target-wrappers-container") as HTMLElement;

		const addTargetBtn = document.getElementById("radar-add-target") as HTMLButtonElement;
		const openSettingsBtn = document.getElementById("radar-open-settings") as HTMLButtonElement;
		const closeSettingsBtn = document.getElementById("radar-close-settings") as HTMLButtonElement;
		const removeAllTargetsBtn = document.getElementById("radar-remove-all-targets") as HTMLElement;
		const updateFrequency = document.getElementById("radar_select_update-frequency") as HTMLSelectElement;

		addTargetBtn.addEventListener("click", () => {
			if (!this.active) return;

			const usernameInput = document.getElementById("radar-target-username") as HTMLInputElement;
			const username = usernameInput.value;

			if (this.targets.has(username)) return;

			usernameInput.value = "";

			this.targets.set(username, { data: null, interval: null, chart: null });

			const card = document.createElement("div");
			card.className = "radar-target";
			card.id = `radar-target-${username}`;

			card.innerHTML = `
        <svg class="card-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-person-fill" viewBox="0 0 16 16">
          <path d="M3 14s-1 0-1-1 1-4 6-4 6 3 6 4-1 1-1 1zm5-6a3 3 0 1 0 0-6 3 3 0 0 0 0 6"/>
        </svg>

        <div class="sep"></div>

        <div class="info" style="min-width: 220px; max-width: 220px;">
          <p>Никнейм: <span>${username.length <= 16 ? username : username.substring(0, 16) + "..."}</span></p>
          <p>Статус: <span id="radar-target-status-${username}">Not found</span></p>
          <p>UUID: <span id="radar-target-uuid-${username}">?</span></p>
        </div>

        <div class="sep"></div>

        <div class="info" style="min-width: 150px; max-width: 150px;">
          <p>X: <span id="radar-target-x-${username}">?</span></p>
          <p>Y: <span id="radar-target-y-${username}">?</span></p>
          <p>Z: <span id="radar-target-z-${username}">?</span></p>
        </div>

        <div class="sep"></div>

        <div class="btn-group">
          <div class="btn-group-flex" style="margin-top: 0;">
            <button class="min" id="radar-open-route-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-route">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M3 19a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" />
                <path d="M19 7a2 2 0 1 0 0 -4a2 2 0 0 0 0 4" />
                <path d="M11 19h5.5a3.5 3.5 0 0 0 0 -7h-8a3.5 3.5 0 0 1 0 -7h4.5" />
              </svg>
            </button>

            <button class="min" id="radar-remove-target-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-trash">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M4 7l16 0" />
                <path d="M10 11l0 6" />
                <path d="M14 11l0 6" />
                <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
                <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
              </svg>
            </button>
          </div>

          <div class="btn-group-flex" style="margin-top: 0;">
            <button class="min" id="radar-follow-target-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-current-location">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M9 12a3 3 0 1 0 6 0a3 3 0 1 0 -6 0" />
                <path d="M4 12a8 8 0 1 0 16 0a8 8 0 1 0 -16 0" />
                <path d="M12 2l0 2" />
                <path d="M12 20l0 2" />
                <path d="M20 12l2 0" />
                <path d="M2 12l2 0" />
              </svg>
            </button>

            <button class="min" id="radar-copy-target-info-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-copy">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M7 9.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667l0 -8.666" />
                <path d="M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1" />
              </svg>
            </button>
          </div>
        </div>
      `;

			const routeWrapper = document.createElement("div");
			routeWrapper.className = "cover";
			routeWrapper.id = `radar-route-${username}`;

			routeWrapper.innerHTML = `
        <div class="panel with-header" style="margin-bottom: 40px;">
          <div class="left">
            <div class="header">Маршрут ${username}</div>
          </div>

          <div class="right">
            <button class="min" id="radar-close-route-${username}">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="icon icon-tabler icons-tabler-outline icon-tabler-x">
                <path stroke="none" d="M0 0h24v24H0z" fill="none" />
                <path d="M18 6l-12 12" />
                <path d="M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <canvas class="radar-chart" id="radar-chart-${username}"></canvas>
      `;

			this.targetCardsContainer?.appendChild(card);
			this.targetWrappersContainer?.appendChild(routeWrapper);

			this.initializeTargetCard(username);
		});

		openSettingsBtn.addEventListener("click", () => (document.getElementById("radar-settings") as HTMLElement).style.display = "flex");
		closeSettingsBtn.addEventListener("click", () => (document.getElementById("radar-settings") as HTMLElement).style.display = "none");

		removeAllTargetsBtn.addEventListener("click", () => {
			this.targets.forEach((v, n) => {
				const card = document.getElementById(`radar-target-${n}`) as HTMLElement;
				v.chart.destroy();
				card.remove();
				clearInterval(v.interval);
			});

			this.targets.clear();
		});

		updateFrequency.addEventListener("change", () => {
			this.updateFrequency = updateFrequency.value ? parseInt(updateFrequency.value) : 1500;

			this.targets.forEach((i, n) => {
				clearInterval(i.interval);
				this.setTargetUpdateInterval(n, this.updateFrequency);
			});
		});

		let lx = "";
		let ly = "";
		let lz = "";

		await listen<number[]>("radar:target-info", async (e) => {
			try {
				const buf = new Buffer(new Uint8Array(e.payload));
				const username = buf.readString();
				const uuid = buf.readString();
				const tx = buf.readF64().toFixed(3);
				const ty = buf.readF64().toFixed(3);
				const tz = buf.readF64().toFixed(3);
				const ox = buf.readF64();
				const oz = buf.readF64();

				(document.getElementById(`radar-target-status-${username}`) as HTMLElement).innerText = "Found";
				(document.getElementById(`radar-target-uuid-${username}`) as HTMLElement).innerText = uuid.substring(0, 12) + "...";
				(document.getElementById(`radar-target-x-${username}`) as HTMLElement).innerText = tx;
				(document.getElementById(`radar-target-y-${username}`) as HTMLElement).innerText = ty;
				(document.getElementById(`radar-target-z-${username}`) as HTMLElement).innerText = tz;

				const target = this.targets.get(username);

				if (target) target.data = { fullUuid: uuid };

				if (lx !== tx || ly !== ty || lz !== tz) {
					const card = document.getElementById(`radar-target-${username}`) as HTMLElement;
					card.classList.add("glow");
					setTimeout(() => card.classList.remove("glow"), 300);
				}

				lx = tx;
				ly = ty;
				lz = tz;

				this.addRoutePointToChart(username, parseFloat(tx), parseFloat(tz), ox, oz);

				if ((document.getElementById("radar_chbx_auto-save") as HTMLInputElement).checked) {
					const path = (document.getElementById("radar_option_path") as HTMLInputElement).value;
					const filename = (document.getElementById("radar_option_filename") as HTMLInputElement).value;

					if (await isAbsolute(path)) {
						const buf = new Buffer();
						buf.writeU8(0x09);
						buf.writeString(username);
						buf.writeString(path);
						buf.writeString(filename || "radar_#t");
						buf.writeF64(parseFloat(tx));
						buf.writeF64(parseFloat(ty));
						buf.writeF64(parseFloat(tz));

						const result = await invoke<CommandResult<null>>("send_command", {
							data: buf.toUint8Array(),
						});

						if (result.error) logger.log(`Ошибка сохранения данных радара об игроке ${username}: ${result.error}`, "error");
					}
				}
			} catch (error) {
				logger.log(`Ошибка обработки события "radar:target-info": ${error}`, "error");
			}
		});
	}

	/** Метод активации радара */
	public enable(): void {
		this.active = true;
	}

	/** Метод выключения радара */
	public disable(): void {
		this.active = false;
	}

	/** Метод инициализации карточки цели */
	private initializeTargetCard(username: string): void {
		try {
			const openRouteBtn = document.getElementById(`radar-open-route-${username}`) as HTMLButtonElement;
			const closeRouteBtn = document.getElementById(`radar-close-route-${username}`) as HTMLButtonElement;
			const removeTargetBtn = document.getElementById(`radar-remove-target-${username}`) as HTMLButtonElement;
			const copyTargetInfoBtn = document.getElementById(`radar-copy-target-info-${username}`) as HTMLButtonElement;
			const followTargetBtn = document.getElementById(`radar-follow-target-${username}`) as HTMLButtonElement;

			openRouteBtn.addEventListener("click", () => (document.getElementById(`radar-route-${username}`) as HTMLElement).style.display = "flex");
			closeRouteBtn.addEventListener("click", () => (document.getElementById(`radar-route-${username}`) as HTMLElement).style.display = "none");

			removeTargetBtn.addEventListener("click", () => {
				const card = document.getElementById(`radar-target-${username}`) as HTMLElement;
				this.targets.get(username)?.chart.destroy();
				card.remove();
				clearInterval(this.targets.get(username)?.interval);
				this.targets.delete(username);
			});

			copyTargetInfoBtn.addEventListener("click", async () => {
				try {
					const status = (document.getElementById(`radar-target-status-${username}`) as HTMLElement).textContent;
					const uuid = this.targets.get(username)?.data?.fullUUID ? this.targets.get(username)?.data.fullUUID : "?";
					const x = (document.getElementById(`radar-target-x-${username}`) as HTMLElement).textContent;
					const y = (document.getElementById(`radar-target-y-${username}`) as HTMLElement).textContent;
					const z = (document.getElementById(`radar-target-z-${username}`) as HTMLElement).textContent;

					const text = `
Никнейм: ${username}
Статус: ${status}
UUID: ${uuid}
Координата X: ${x}
Координата Y: ${y}
Координата Z: ${z}
          `.trim();

					await navigator.clipboard.writeText(text);

					messages.message("Радар", `Данные игрока ${username} успешно скопированы в буфер обмена`);
				} catch (error) {
					logger.log(`Ошибка копирования данных радара: ${error}`, "error");
				}
			});

			followTargetBtn.addEventListener("click", async () => {
				try {
					const xText = document.getElementById(`radar-target-x-${username}`)?.textContent;
					const zText = document.getElementById(`radar-target-z-${username}`)?.textContent;

					if (!xText || !zText) return;

					const x = parseInt(xText);
					const z = parseInt(zText);

					const buf = new Buffer();
					buf.writeU8(0x0A);
					buf.writeString(username);
					buf.writeI32(x);
					buf.writeI32(z);

					const result = await invoke<CommandResult<null>>("send_command", {
						data: buf.toUint8Array(),
					});

					if (result.error) logger.log(`Ошибка преследования ${username}: ${result.error}`, "error");
				} catch (error) {
					logger.log(`Ошибка преследования ${username}: ${error}`, "error");
				}
			});

			this.targets.set(username, { data: null, interval: null, chart: null });

			this.createTargetChart(username);
			this.setTargetUpdateInterval(username, this.updateFrequency);
		} catch (error) {
			logger.log(`Ошибка инициализации цели радара: ${error}`, "error");
		}
	}

	/** Метод установки частоты обновления */
	private setTargetUpdateInterval(username: string, frequency: number) {
		const target = this.targets.get(username);

		if (!target) return;

		target.interval = setInterval(async () => {
			if (!this.active) {
				clearInterval(target.interval);
				return;
			}

			try {
				const buf = new Buffer();
				buf.writeU8(0x08);
				buf.writeU32(2 + username.length);
				buf.writeString(username);

				const result = await invoke<CommandResult<null>>("send_command", {
					data: buf.toUint8Array(),
				});

				if (result.error) {
					logger.log(`Ошибка обновления цели радара ${username}: ${result.error}`, "error");
					clearInterval(target.interval);
					return;
				}
			} catch (error) {
				logger.log(`Ошибка обновления цели радара ${username}: ${error}`, "error");
			}
		}, frequency);
	}

	/** Метод создания обёртки маршрута цели */
	private createTargetChart(username: string) {
		const ctx = document.getElementById(`radar-chart-${username}`) as HTMLCanvasElement;

		const chart = new Chart(ctx, {
			type: "scatter",
			data: {
				datasets: [
					{
						label: ` Маршрут ${username}`,
						data: [],
						backgroundColor: "#39a10fff",
						borderColor: "#0f8f0bff",
						showLine: true,
						fill: false,
						pointRadius: 2,
						tension: 0,
						borderWidth: 2
					},
					{
						label: ` Метка наблюдателя`,
						data: [],
						backgroundColor: "#d31212ff",
						borderColor: "#800c0cff",
						showLine: false,
						fill: false,
						pointRadius: 3,
						tension: 0,
						borderWidth: 1
					}
				]
			},
			options: {
				responsive: true,
				animation: {
					duration: 300
				},
				scales: {
					x: {
						type: "linear",
						position: "bottom",
						title: {
							display: true,
							text: "X"
						},
						min: -200,
						max: 200,
						grid: {
							color: "#30303086"
						},
						ticks: {
							stepSize: 50
						}
					},
					y: {
						type: "linear",
						position: "left",
						title: {
							display: true,
							text: "Z"
						},
						min: -200,
						max: 200,
						grid: {
							color: "#30303086"
						},
						ticks: {
							stepSize: 50
						}
					}
				},
				plugins: {
					title: {
						display: false,
					},
					legend: {
						display: false
					},
					tooltip: {
						enabled: false
					}
				}
			}
		});

		const target = this.targets.get(username);

		if (target) target.chart = chart;
	}

	/** Метод добавления поинта цели (её текущей позиции) на чарт маршрута */
	private addRoutePointToChart(username: string, tx: number, tz: number, ox: number, oz: number) {
		const target = this.targets.get(username);

		if (!target) return;

		target.chart.data.datasets[0].data.push({ x: tx, y: tz });
		target.chart.data.datasets[1].data.push({ x: ox, y: oz });

		if (target.chart.data.datasets[0].data.length > 30) target.chart.data.datasets[0].data.shift();
		if (target.chart.data.datasets[1].data.length > 1) target.chart.data.datasets[1].data.shift();

		const xMin = Number(tx.toFixed(1)) - 200;
		const xMax = Number(tx.toFixed(1)) + 200;
		const zMin = Number(tz.toFixed(1)) - 200;
		const zMax = Number(tz.toFixed(1)) + 200;

		target.chart.options.scales.x.min = xMin;
		target.chart.options.scales.x.max = xMax;
		target.chart.options.scales.y.min = zMin;
		target.chart.options.scales.y.max = zMax;

		target.chart.update();
	}
}

const radarModule = new RadarModule();

export { radarModule }
