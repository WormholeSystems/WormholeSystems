import DOMPurify from 'dompurify';
import { marked } from 'marked';

/**
 * Markdown somebody typed, as HTML that is safe to hand to `{@html}`. marked passes raw
 * HTML and `javascript:` links straight through, so everything it produces goes through
 * DOMPurify, which drops scripts, event handlers and unsafe URLs and keeps the rest.
 *
 * Browser only: DOMPurify needs a DOM, and without one it would return its input
 * untouched. On the server this renders nothing rather than something unsanitized.
 */
export function renderMarkdown(markdown: string, options: { breaks?: boolean } = {}): string {
	if (!DOMPurify.isSupported) return '';
	// SAFETY: `marked.parse` returns a promise only when `async` is set, and it is not.
	const html = marked.parse(markdown, { async: false, breaks: options.breaks }) as string;
	return DOMPurify.sanitize(html);
}

// Every link leaves the app in a new tab, without handing it this page through `opener`
// or the URL it was on through the referrer.
DOMPurify.addHook('afterSanitizeAttributes', (node) => {
	if (node.tagName === 'A' && node.hasAttribute('href')) {
		node.setAttribute('target', '_blank');
		node.setAttribute('rel', 'noopener noreferrer');
	}
});
