/**
 * Static TypeScript module for signature categories and types.
 *
 * resources/js/data/signatures.json is the single source of truth for this data.
 * The frontend imports it directly here, and the database is seeded from the same
 * file via SignatureCategorySeeder / SignatureTypeSeeder (run `php artisan db:seed`).
 */

import signatureData from '@/data/signatures.json';
import { TSignatureCategory, TSignatureType } from '@/types/models';

// Raw data from JSON
export const signatureCategories: TSignatureCategory[] = signatureData.categories as TSignatureCategory[];
export const signatureTypes: TSignatureType[] = signatureData.types as TSignatureType[];

// Lookup maps for faster access
export const signatureCategoryById = new Map(signatureCategories.map((cat) => [cat.id, cat]));

export const signatureCategoryByName = new Map(signatureCategories.map((cat) => [cat.name, cat]));

export const signatureCategoryByCode = new Map(signatureCategories.map((cat) => [cat.code, cat]));

export const signatureTypeById = new Map(signatureTypes.map((type) => [type.id, type]));

export const signatureTypesByCategory = signatureTypes.reduce(
    (acc, type) => {
        if (!acc[type.signature_category_id]) {
            acc[type.signature_category_id] = [];
        }
        acc[type.signature_category_id].push(type);
        return acc;
    },
    {} as Record<number, TSignatureType[]>,
);

export const signatureTypesByCategoryName = signatureTypes.reduce(
    (acc, type) => {
        if (!acc[type.category_name]) {
            acc[type.category_name] = [];
        }
        acc[type.category_name].push(type);
        return acc;
    },
    {} as Record<string, TSignatureType[]>,
);

// Helper functions
export function getCategoryById(id: number): TSignatureCategory | undefined {
    return signatureCategoryById.get(id);
}

export function getCategoryByName(name: string): TSignatureCategory | undefined {
    return signatureCategoryByName.get(name);
}

export function getTypeById(id: number): TSignatureType | undefined {
    return signatureTypeById.get(id);
}

export function getTypesByCategory(categoryId: number): TSignatureType[] {
    return signatureTypesByCategory[categoryId] || [];
}

export function getTypesByCategoryName(categoryName: string): TSignatureType[] {
    return signatureTypesByCategoryName[categoryName] || [];
}
