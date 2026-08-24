import { invoke } from "@tauri-apps/api/core";

import { logger } from "../utils/logger";
import { generateId } from "../utils/generator";

/** Структура информации сервера */
interface ServerInformation {
	ip_address: string;
	server_icon: string | null;
	protocol_version: number;
	server_version: string;
	description: string;
	players_online: number;
	players_max: number;
	list_of_players: Array<{ username: string; uuid: string; }>;
}

/** Модуль управления пинговкой серверов */
class PingModule {
	/** Метод инициализации функций, связанных с пингованием */
	public init(): void {
		document.getElementById("ping-server")?.addEventListener("click", async () => await this.ping_server());
	}

	/** Метод пингования сервера */
	private async ping_server(): Promise<void> {
		try {
			const address = (document.getElementById("ping-server-address") as HTMLInputElement).value;

			if (address === "") return;

			const result = await invoke<ServerInformation>("get_server_info", {
				address: address
			});

			const pingInfo = document.getElementById("ping-info") as HTMLElement;
			pingInfo.innerHTML = "";

			const card = document.createElement("div");
			card.className = "card";

			const removeBtnId = `remove-ping-card-${generateId()}`;

			card.innerHTML = `
        <img class="icon" src="${result.server_icon}" draggable="false">
        <div class="text">
          <label>${result.description}</label>
          <div>
            <p>Игроки: ${result.players_online} / ${result.players_max}</p>
            <p>Версия сервера: ${result.server_version}</p>
            <p>IP-адрес: ${result.ip_address}</p>
            <p>Версия протокола: ${result.protocol_version}</p>
          </div>
        </div>

        <button class="min" id="${removeBtnId}">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none" />
            <path d="M18 6l-12 12" />
            <path d="M6 6l12 12" />
          </svg>
        </button>
      `;

			pingInfo.appendChild(card);

			const listOfPlayers = document.createElement("div");
			listOfPlayers.className = "list";
			listOfPlayers.style.display = "none";

			if (result.list_of_players.length > 0) {
				listOfPlayers.style.display = "flex";

				const element = document.createElement("div");
				element.className = "element";

				element.innerHTML = `
          <p class="username">Никнейм</p>
          <div class="splitter"></div>
          <p class="uuid">UUID</p>
        `;

				listOfPlayers.appendChild(element);

				for (const player of result.list_of_players) {
					const el = document.createElement("div");
					el.className = "element";

					el.innerHTML = `
            <p class="username">${player.username}</p>
            <div class="splitter"></div>
            <p class="uuid">${player.uuid}</>
          `;

					listOfPlayers.appendChild(el);
				}

				pingInfo.appendChild(listOfPlayers);
			} else {
				const header = document.createElement("div");
				header.className = "header";
				header.innerText = "Не удалось получить список игроков";
				pingInfo.appendChild(header);
			}

			document.getElementById(removeBtnId)?.addEventListener("click", () => {
				card.remove();
				listOfPlayers.remove();
				pingInfo.innerHTML = "";
			});
		} catch (error) {
			logger.log(`Ошибка пингования сервера: ${error}`, "error");
		}
	}
}

const pingModule = new PingModule();

export { pingModule }
