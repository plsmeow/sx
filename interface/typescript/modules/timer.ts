import { messages } from "../utils/message";

/** Неюольшой модуль отображения времени использования */
class TimerModule {
	/** Метод инициализации таймера */
	public init(): void {
		let playTime = document.getElementById("play-time") as HTMLLabelElement;
		let hours = 0;
		let minutes = 0;
		let seconds = 0;

		setInterval(() => {
			seconds++;

			if (seconds >= 60) {
				minutes++;
				seconds = 0;
			}

			if (minutes >= 60) {
				hours++;
				minutes = 0;
			}

			let time = this.createTimeString(hours, minutes, seconds);
			playTime.innerText = time;

			switch (time) {
				case "30:00":
					messages.message("salarixi", `Ого, ты активен уже 30 минут! Не хочешь передохнуть?`);
					break;
				case "01:00:00":
					messages.message("salarixi", `Надо же, ты активен уже 1 час! Не хочешь передохнуть?`);
					break;
				case "02:00:00":
					messages.message("salarixi", `Ничего себе! Ты активен уже 2 часа, не хочешь передохнуть?`);
					break;
			}
		}, 1000);
	}

	/** Метод создания строки времени */
	private createTimeString(hours: number, minutes: number, seconds: number): string {
		if (hours === 0) {
			if (minutes === 0) {
				return `00:${seconds.toString().padStart(2, "0")}`;
			} else {
				return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
			}
		} else {
			return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
		}
	}
}

const timerModule = new TimerModule();

export { timerModule };
