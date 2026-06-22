<?php

declare(strict_types=1);

use App\Enums\SolarsystemClass;
use App\Enums\WormholeSignature;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use Database\Seeders\SignatureCategorySeeder;
use Database\Seeders\SignatureTypeSeeder;

/**
 * resources/js/data/signatures.json is the single source of truth: the frontend
 * imports it and the database is seeded from it. These tests guard that the DB
 * mirrors the file exactly (ids included) and that the file stays internally
 * consistent.
 */
function signatureSourceData(): array
{
    return json_decode(file_get_contents(resource_path('js/data/signatures.json')), true, flags: JSON_THROW_ON_ERROR);
}

it('seeds every category from the source file with matching ids', function (): void {
    $categories = signatureSourceData()['categories'];

    expect(SignatureCategory::query()->count())->toBe(count($categories));

    foreach ($categories as $category) {
        $model = SignatureCategory::query()->find($category['id']);

        expect($model)->not->toBeNull()
            ->and($model->name)->toBe($category['name'])
            ->and($model->code->value)->toBe($category['code']);
    }
});

it('seeds every type from the source file with matching ids and fields', function (): void {
    $types = signatureSourceData()['types'];

    expect(SignatureType::query()->count())->toBe(count($types));

    foreach ($types as $type) {
        $model = SignatureType::query()->find($type['id']);

        expect($model)->not->toBeNull()
            ->and($model->name)->toBe($type['name'])
            ->and($model->signature_category_id)->toBe($type['signature_category_id'])
            ->and($model->signature?->value)->toBe($type['signature'])
            ->and($model->target_class?->value)->toBe($type['target_class'])
            ->and($model->extra)->toBe($type['extra'])
            ->and(array_map(fn (SolarsystemClass $c): string => $c->value, $model->spawn_areas ?? []))
            ->toBe($type['spawn_areas'] ?? []);
    }
});

it('spot checks a known wormhole type resolves correctly', function (): void {
    $type = SignatureType::query()
        ->where('name', 'A009 - C13')
        ->firstOrFail();

    expect($type->signature)->toBe(WormholeSignature::A009)
        ->and($type->target_class)->toBe(SolarsystemClass::C13)
        ->and($type->category->code->value)->toBe('wormhole');
});

it('keeps the source file internally consistent', function (): void {
    $data = signatureSourceData();

    $categoryIds = array_column($data['categories'], 'id');
    $typeIds = array_column($data['types'], 'id');
    $typeNames = array_column($data['types'], 'name');

    expect($categoryIds)->toEqual(array_unique($categoryIds))
        ->and($typeIds)->toEqual(array_unique($typeIds))
        ->and($typeNames)->toEqual(array_unique($typeNames));

    foreach ($data['types'] as $type) {
        expect($categoryIds)->toContain($type['signature_category_id']);

        if ($type['signature'] !== null) {
            expect(WormholeSignature::tryFrom($type['signature']))->not->toBeNull();
        }

        if ($type['target_class'] !== null) {
            expect(SolarsystemClass::tryFrom($type['target_class']))->not->toBeNull();
        }

        foreach ($type['spawn_areas'] ?? [] as $area) {
            expect(SolarsystemClass::tryFrom($area))->not->toBeNull();
        }
    }
});

it('is idempotent when reseeded', function (): void {
    $categories = SignatureCategory::query()->count();
    $types = SignatureType::query()->count();

    (new SignatureCategorySeeder)->run();
    (new SignatureTypeSeeder)->run();

    expect(SignatureCategory::query()->count())->toBe($categories)
        ->and(SignatureType::query()->count())->toBe($types);
});
