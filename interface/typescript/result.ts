/** Структура результата выполнения команды */
interface CommandResult<T> {
	data: T | null;
	error: string | null;
}
