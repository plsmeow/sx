import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";

/** Модуль управления графиком на верхней панели */
class ChartModule {
	private maxBarCount: number = 80;
	private ramUsageBars: HTMLDivElement[] = [];
	private ramUsageChart: HTMLDivElement | null = null;
	private ramUsageData: Array<number> = [];
	private cpuUsageBars: HTMLDivElement[] = [];
	private cpuUsageChart: HTMLDivElement | null = null;
	private cpuUsageData: Array<number> = [];

	/** Метод инициализации модуля */
	public init(): void {
		this.ramUsageChart = document.getElementById("ram-usage-chart") as HTMLDivElement;
		this.cpuUsageChart = document.getElementById("cpu-usage-chart") as HTMLDivElement;

		for (let i = 0; i < this.maxBarCount; i++) {
			const ramBar = document.createElement("div");
			ramBar.className = "bar";
			this.ramUsageChart.appendChild(ramBar);
			this.ramUsageBars.push(ramBar);

			const cpuBar = document.createElement("div");
			cpuBar.className = "bar";
			this.cpuUsageChart.appendChild(cpuBar);
			this.cpuUsageBars.push(cpuBar);
		}

		const ramUsageSpan = document.getElementById("current-ram-usage") as HTMLSpanElement;
		const cpuUsageSpan = document.getElementById("current-cpu-usage") as HTMLSpanElement;

		setInterval(async () => {
			try {
				const data = await invoke<number>("get_ram_usage");
				const usage = parseFloat(data.toFixed(1));
				ramUsageSpan.innerText = `${usage}MB`;
				this.updateChart("ram", usage > 1024.0 ? 1024.0 : usage);
			} catch (error) {
				logger.log(`Ошибка графика RAM: ${error}`, "error");
			}
		}, 500);

		setInterval(async () => {
			try {
				const data = await invoke<number>("get_cpu_usage");
				const usage = parseFloat(data.toFixed(1));
				cpuUsageSpan.innerText = `${usage}%`;
				this.updateChart("cpu", usage);
			} catch (error) {
				logger.log(`Ошибка графика CPU: ${error}`, "error");
			}
		}, 1000);
	}

	/** Метод обновления графика */
	private updateChart(chart: "ram" | "cpu", value: number): void {
		const data = chart === "ram" ? this.ramUsageData : this.cpuUsageData;
		const bars = chart === "ram" ? this.ramUsageBars : this.cpuUsageBars;

		data.push(value);
		if (data.length > this.maxBarCount) data.shift();

		const padded = new Array(this.maxBarCount - data.length).fill(0).concat(data);
		const maxValue = chart === "ram" ? 1024 : 100;

		for (let i = 0; i < this.maxBarCount; i++) {
			const height = Math.round((Math.min(maxValue, Math.max(0, padded[i])) - 0) / (maxValue - 0) * 20);
			bars[i].style.height = `${height}px`;
		}
	}
}

const chartModule = new ChartModule();

export { chartModule }
