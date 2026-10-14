// The layout-as-a-string you can hand to someone else: base64 over JSON, validated on the
// way back in because the clipboard is untrusted input.

import * as v from 'valibot';

import { layoutClipboardSchema, type PanelLayouts } from './registry';

export interface LayoutClipboard {
	breakpoints: PanelLayouts;
	hidden?: string[];
}

export function encodeLayout(payload: LayoutClipboard): string {
	return btoa(JSON.stringify(payload));
}

/** Null for anything that is not a pasted layout; the caller words the complaint. */
export function decodeLayout(text: string): LayoutClipboard | null {
	let parsed: unknown;
	try {
		parsed = JSON.parse(atob(text.trim()));
	} catch {
		return null;
	}
	const result = v.safeParse(layoutClipboardSchema, parsed);
	return result.success ? result.output : null;
}
