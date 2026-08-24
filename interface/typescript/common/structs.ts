/** Локальная структура, содержащая информацию о плагинах текущей версии программы */
export const plugins: Record<string, { name: string; enable: boolean; date: string }> = {
	"instant-armor-equip": {
		name: "Instant Armor Equip",
		enable: false,
		date: "28.07.2026"
	},
	"auto-totem": {
		name: "Auto Totem",
		enable: false,
		date: "29.07.2026"
	},
	"auto-eat": {
		name: "Auto Eat",
		enable: false,
		date: "14.08.2026"
	},
	"potion-consumer": {
		name: "Potion Consumer",
		enable: false,
		date: "14.08.2026"
	},
	"auto-look": {
		name: "Auto Look",
		enable: false,
		date: "29.07.2026"
	},
	"auto-shield": {
		name: "Auto Shield",
		enable: false,
		date: "29.07.2026"
	},
	"auto-mending": {
		name: "Auto Mending",
		enable: false,
		date: "04.07.2026"
	},
	"pearl-leave": {
		name: "Pearl Leave",
		enable: false,
		date: "14.08.2026"
	},
};
