// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';

import { renderMarkdown } from './markdown';

/** The rendered HTML as a document fragment, to ask about elements rather than strings. */
function parse(html: string): HTMLElement {
	const div = document.createElement('div');
	div.innerHTML = html;
	return div;
}

describe('renderMarkdown', () => {
	it('drops event handlers from raw HTML', () => {
		const out = parse(renderMarkdown('<img src=x onerror="alert(1)"> hi'));
		const img = out.querySelector('img');
		expect(img).not.toBeNull();
		expect(img!.hasAttribute('onerror')).toBe(false);
		expect(out.innerHTML).not.toContain('alert');
	});

	it('drops scripts', () => {
		expect(renderMarkdown('<script>alert(1)</script>ok')).not.toContain('script');
	});

	it('drops javascript: links but keeps their text', () => {
		const out = parse(renderMarkdown('[click](javascript:alert(1))'));
		const link = out.querySelector('a');
		expect(link?.getAttribute('href') ?? null).toBeNull();
		expect(out.textContent).toContain('click');
		expect(out.innerHTML).not.toContain('javascript:');
	});

	it('keeps ordinary markdown', () => {
		const out = parse(renderMarkdown('# Title\n\n**bold** and `code`\n\n- one\n- two'));
		expect(out.querySelector('h1')?.textContent).toBe('Title');
		expect(out.querySelector('strong')?.textContent).toBe('bold');
		expect(out.querySelector('code')?.textContent).toBe('code');
		expect(out.querySelectorAll('li')).toHaveLength(2);
	});

	it('opens links in a new tab without opener or referrer', () => {
		const link = parse(renderMarkdown('[evewho](https://evewho.com/)')).querySelector('a')!;
		expect(link.getAttribute('href')).toBe('https://evewho.com/');
		expect(link.getAttribute('target')).toBe('_blank');
		expect(link.getAttribute('rel')).toBe('noopener noreferrer');
	});

	it('turns single newlines into breaks when asked', () => {
		expect(parse(renderMarkdown('a\nb', { breaks: true })).querySelector('br')).not.toBeNull();
		expect(parse(renderMarkdown('a\nb')).querySelector('br')).toBeNull();
	});
});
