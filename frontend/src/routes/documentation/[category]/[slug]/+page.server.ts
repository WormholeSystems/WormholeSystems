import { error } from '@sveltejs/kit';
import { findPage, neighbours, render } from '$lib/docs';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
	const page = findPage(params.category, params.slug);
	if (!page) error(404, 'No such documentation page');
	return {
		page,
		html: render(page.markdown),
		...neighbours(page),
		seo: { title: `${page.title} · Documentation`, description: summary(page.markdown) },
	};
};

/** The first paragraph, flattened, for the card that shares the page. */
function summary(markdown: string): string {
	const prose = markdown
		.split(/\r?\n\r?\n/)
		.map((block) => block.trim())
		.find((block) => block && !block.startsWith('#') && !block.startsWith('>'));
	if (!prose) return 'Documentation for WormholeSystems.';
	const flat = prose
		.replace(/[*_`]/g, '')
		.replace(/\[(.+?)\]\(.+?\)/g, '$1')
		.replace(/\s+/g, ' ');
	return flat.length > 180 ? `${flat.slice(0, 177)}…` : flat;
}
