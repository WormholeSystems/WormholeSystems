// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		/**
		 * What an error page is handed. `message` is always something written for a person,
		 * never the text of whatever was thrown.
		 */
		interface Error {
			message: string;
			/** Which of the error page's explanations fits, where the status alone is not enough. */
			kind?: 'map' | 'share' | 'offline';
			/** A short id to quote: the backend's request id, or one logged with the crash. */
			errorId?: string;
		}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
