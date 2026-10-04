<?php

declare(strict_types=1);

use App\Enums\SignatureCategory;
use App\Enums\SolarsystemClass;
use App\Models\SignatureType;
use Illuminate\Database\Migrations\Migration;

function rampantDroneFabricatorMigration(): Migration
{
    return require database_path('migrations/2026_10_04_104344_add_rampant_drone_fabricator_signature_type.php');
}

it('offers the Rampant Drone Fabricator as a combat site in C1 to C6', function (): void {
    $type = SignatureType::query()->where('name', 'Rampant Drone Fabricator')->firstOrFail();

    expect($type->id)->toBe(272)
        ->and($type->category->code)->toBe(SignatureCategory::Combat)
        ->and($type->signature)->toBeNull()
        ->and($type->target_class)->toBeNull()
        ->and($type->spawn_areas)->toBe([
            SolarsystemClass::C1,
            SolarsystemClass::C2,
            SolarsystemClass::C3,
            SolarsystemClass::C4,
            SolarsystemClass::C5,
            SolarsystemClass::C6,
        ]);
});

it('adds the Rampant Drone Fabricator to databases seeded before it existed', function (): void {
    $migration = rampantDroneFabricatorMigration();
    $seeded = SignatureType::query()->findOrFail(272)->only(['name', 'signature_category_id', 'spawn_areas']);

    $migration->down();

    expect(SignatureType::query()->find(272))->toBeNull();

    $migration->up();

    expect(SignatureType::query()->findOrFail(272)->only(['name', 'signature_category_id', 'spawn_areas']))
        ->toEqual($seeded);
});

it('leaves an already seeded Rampant Drone Fabricator untouched', function (): void {
    $count = SignatureType::query()->count();

    rampantDroneFabricatorMigration()->up();

    expect(SignatureType::query()->count())->toBe($count);
});
