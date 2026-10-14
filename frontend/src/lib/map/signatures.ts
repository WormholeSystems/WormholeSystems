// Signature catalog access, category metadata, and the scanner-paste parser. The catalog
// comes from `/api/signature-types` and is cached for the session.

import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
import CloudIcon from '@lucide/svelte/icons/cloud';
import DatabaseIcon from '@lucide/svelte/icons/database';
import FanIcon from '@lucide/svelte/icons/fan';
import FlagIcon from '@lucide/svelte/icons/flag';
import GemIcon from '@lucide/svelte/icons/gem';
import LandmarkIcon from '@lucide/svelte/icons/landmark';
import ShieldIcon from '@lucide/svelte/icons/shield';
import SwordsIcon from '@lucide/svelte/icons/swords';

import type { QueryClient } from '@tanstack/svelte-query';

import { q } from '$lib/api/queries';
import type { PastedSignature } from '$lib/api/types/PastedSignature';
import type { Signature } from '$lib/api/types/Signature';
import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
import type { SignatureGroup } from '$lib/api/types/SignatureGroup';
import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
import translations from '$lib/signatures/translations.json';
import { destClassMeta } from './classes';

// Localized category and site names from the EVE client's localization files, mapped to
// their English catalog names. Site names are keyed by English category, because one
// localized name can mean different sites in different categories. A localized name that
// several sites share is left out, so it stays a raw name rather than a wrong guess.
const localizedCategories: Record<string, string> = translations.categories;
const localizedTypes: Record<string, Record<string, string>> = translations.types;

export interface CategoryMeta {
	group: SignatureGroup;
	categoryId: number | null;
	label: string;
	abbrev: string;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	icon: any;
	color: string;
}

/** Legacy category vocabulary, in the legacy filter order. `unknown` = uncategorized. */
export const CATEGORIES: CategoryMeta[] = [
	{
		group: 'wormhole',
		categoryId: 1,
		label: 'Wormhole',
		abbrev: 'WH',
		icon: FanIcon,
		color: 'text-sky-400',
	},
	{
		group: 'data',
		categoryId: 2,
		label: 'Data Site',
		abbrev: 'Data',
		icon: DatabaseIcon,
		color: 'text-cyan-400',
	},
	{
		group: 'relic',
		categoryId: 3,
		label: 'Relic Site',
		abbrev: 'Relic',
		icon: LandmarkIcon,
		color: 'text-amber-400',
	},
	{
		group: 'ore',
		categoryId: 6,
		label: 'Ore Site',
		abbrev: 'Ore',
		icon: GemIcon,
		color: 'text-yellow-400',
	},
	{
		group: 'gas',
		categoryId: 5,
		label: 'Gas Site',
		abbrev: 'Gas',
		icon: CloudIcon,
		color: 'text-orange-400',
	},
	{
		group: 'combat',
		categoryId: 4,
		label: 'Combat Site',
		abbrev: 'Combat',
		icon: SwordsIcon,
		color: 'text-green-400',
	},
	{
		group: 'homefront',
		categoryId: 7,
		label: 'Homefront Operations',
		abbrev: 'HF',
		icon: ShieldIcon,
		color: 'text-rose-400',
	},
	{
		group: 'faction_warfare',
		categoryId: 8,
		label: 'Factional Warfare Site',
		abbrev: 'FW',
		icon: FlagIcon,
		color: 'text-fuchsia-400',
	},
	{
		group: 'unknown',
		categoryId: null,
		label: 'Uncategorized',
		abbrev: '—',
		icon: CircleHelpIcon,
		color: 'text-muted-foreground',
	},
];

export function categoryMeta(group: SignatureGroup): CategoryMeta {
	return CATEGORIES.find((c) => c.group === group) ?? CATEGORIES[CATEGORIES.length - 1];
}

export function groupForCategoryId(categoryId: number): SignatureGroup {
	return CATEGORIES.find((c) => c.categoryId === categoryId)?.group ?? 'unknown';
}

/** The signature type catalog, cached forever in the query client. */
export function loadCatalog(client: QueryClient): Promise<SignatureCatalog> {
	return client.ensureQueryData(q.signatureCatalog());
}

/**
 * Types of one category that can spawn in a system of the given class, sorted by destination
 * class (wormholes) with sites left in catalog order.
 */
export function typesForCategory(
	catalog: SignatureCatalog,
	categoryId: number,
	systemClassId: number | null,
): SignatureTypeInfo[] {
	return catalog.types
		.filter(
			(t) =>
				t.signature_category_id === categoryId &&
				(systemClassId === null || t.spawn_areas.includes(systemClassId)),
		)
		.toSorted(
			(a, b) => destClassMeta(a.target_class).sortWeight - destClassMeta(b.target_class).sortWeight,
		);
}

export function typeById(catalog: SignatureCatalog, id: number | null): SignatureTypeInfo | null {
	return id === null ? null : (catalog.types.find((t) => t.id === id) ?? null);
}

/**
 * Parse a pasted in-game probe scan. Tab-separated rows of `id, scan group (ignored),
 * category, type name, signal %, distance`; rows under 4 columns or without a 7-char id are
 * skipped. Category matches by exact name then by ` - ` segments (first known segment wins).
 * Type matches by exact name within the category, but never for wormholes, whose type the user
 * picks rather than the scanner. An unmatched type name is kept as the raw `name`.
 *
 * Names from any client language match: localized categories and sites are mapped to their
 * English names first, so a clipboard mixing rows from different clients parses the same.
 */
export function parseScan(text: string, catalog: SignatureCatalog): PastedSignature[] {
	const out: PastedSignature[] = [];
	for (const line of text.split('\n')) {
		const cols = line.split('\t');
		if (cols.length < 4) continue;
		const sid = (cols[0] ?? '').trim();
		if (sid.length !== 7) continue;

		const category = matchCategory(catalog, normalizeName(cols[2] ?? ''));
		const typeName = normalizeName(cols[3] ?? '');
		let typeId: number | null = null;
		if (category !== null && groupForCategoryId(category) !== 'wormhole' && typeName) {
			const categoryName = catalog.categories.find((c) => c.id === category)?.name ?? '';
			const englishName = localizedTypes[categoryName]?.[typeName] ?? typeName;
			typeId =
				catalog.types.find((t) => t.signature_category_id === category && t.name === englishName)
					?.id ?? null;
		}
		out.push({
			signature_id: sid,
			group: category === null ? undefined : groupForCategoryId(category),
			signature_type_id: typeId ?? undefined,
			name: category !== null && typeId === null && typeName ? typeName : undefined,
		});
	}
	return out;
}

/** A scanner id as typed: separators stripped, uppercased, hyphenated once long enough. */
export function formatSignatureId(raw: string): string {
	const clean = raw.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
	return clean.length >= 4 ? `${clean.slice(0, 3)}-${clean.slice(3, 6)}` : clean;
}

// Wormhole ages run from creation; site ages from the last update.
function modifiedDate(s: Signature): number {
	return Date.parse(s.group === 'wormhole' ? s.created_at : s.updated_at);
}

function cmpNullableStrings(a: string | null, b: string | null): number {
	if (a === null && b === null) return 0;
	if (a === null) return 1;
	if (b === null) return -1;
	return a.localeCompare(b);
}

/**
 * The signatures panel's per-column comparison, before the sort direction is applied.
 * `typeName` resolves a signature's display name, which needs the loaded catalog.
 */
export function compareSignatures(
	a: Signature,
	b: Signature,
	column: 'id' | 'category' | 'type' | 'age',
	typeName: (s: Signature) => string | null,
): number {
	switch (column) {
		case 'id':
			return a.signature_id.localeCompare(b.signature_id);
		case 'category':
			return cmpNullableStrings(
				a.group === 'unknown' ? null : a.group,
				b.group === 'unknown' ? null : b.group,
			);
		case 'type':
			return cmpNullableStrings(typeName(a), typeName(b));
		case 'age':
			// Newest first in ascending order.
			return modifiedDate(b) - modifiedDate(a);
	}
}

/**
 * Some client languages separate words with non-breaking spaces, so names are compared on
 * collapsed whitespace.
 */
function normalizeName(name: string): string {
	return name.replace(/\s+/g, ' ').trim();
}

function findCategory(catalog: SignatureCatalog, name: string): number | null {
	const english = localizedCategories[name] ?? name;
	return catalog.categories.find((c) => c.name === english)?.id ?? null;
}

function matchCategory(catalog: SignatureCatalog, name: string): number | null {
	if (!name) return null;
	const exact = findCategory(catalog, name);
	if (exact !== null) return exact;
	for (const segment of name.split(' - ')) {
		const hit = findCategory(catalog, segment.trim());
		if (hit !== null) return hit;
	}
	return null;
}
