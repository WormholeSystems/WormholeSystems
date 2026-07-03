<?php

declare(strict_types=1);

use App\Models\Category;
use App\Models\Group;
use App\Models\Type;
use App\Models\User;

use function Pest\Laravel\actingAs;
use function Pest\Laravel\getJson;

function seedShipReferenceData(): void
{
    Category::query()->create(['id' => 6, 'name' => 'Ship']);
    Category::query()->create(['id' => 9, 'name' => 'Blueprint']);

    Group::query()->create(['id' => 27, 'name' => 'Battleship', 'category_id' => 6]);
    Group::query()->create(['id' => 26, 'name' => 'Cruiser', 'category_id' => 6]);
    Group::query()->create(['id' => 100, 'name' => 'Battleship Blueprint', 'category_id' => 9]);

    Type::query()->create(['id' => 638, 'name' => 'Raven', 'group_id' => 27]);
    Type::query()->create(['id' => 16229, 'name' => 'Rokh', 'group_id' => 27]);
    Type::query()->create(['id' => 700, 'name' => 'Raven Blueprint', 'group_id' => 100]);
}

it('searches types by name across all categories', function () {
    seedShipReferenceData();
    actingAs(User::factory()->create());

    $response = getJson(route('eve.ship-search', ['kind' => 'type', 'q' => 'Raven']));

    $response->assertOk();
    $ids = collect($response->json('data'))->pluck('id');

    // Both the ship and the (non-ship) blueprint match — not restricted to ships.
    expect($ids)->toContain(638)
        ->and($ids)->toContain(700)
        ->and($response->json('data.0.group_name'))->toBe('Battleship');
});

it('searches groups by name across all categories and includes the category name', function () {
    seedShipReferenceData();
    actingAs(User::factory()->create());

    $response = getJson(route('eve.ship-search', ['kind' => 'group', 'q' => 'Battleship']));

    $response->assertOk();
    $data = collect($response->json('data'));

    expect($data->pluck('id'))->toContain(27)->toContain(100)
        ->and($data->firstWhere('id', 27)['category_name'])->toBe('Ship');
});

it('includes unpublished entries such as NPC/officer types', function () {
    seedShipReferenceData();
    // NPC/officer entities are unpublished in the SDE but still appear on killmails.
    Type::query()->create(['id' => 999, 'name' => 'Raven State Issue', 'group_id' => 27, 'published' => false]);
    actingAs(User::factory()->create());

    $ids = collect(getJson(route('eve.ship-search', ['kind' => 'type', 'q' => 'Raven']))->json('data'))->pluck('id');

    expect($ids)->toContain(999);
});

it('ranks an exact match first even when many partial matches sort earlier', function () {
    seedShipReferenceData();
    // 30 partial matches that all sort alphabetically before the exact "Capsule".
    for ($i = 1; $i <= 30; $i++) {
        Type::query()->create(['id' => 50000 + $i, 'name' => sprintf('Booster Capsule %02d', $i), 'group_id' => 27]);
    }
    Type::query()->create(['id' => 670, 'name' => 'Capsule', 'group_id' => 27]);
    actingAs(User::factory()->create());

    $response = getJson(route('eve.ship-search', ['kind' => 'type', 'q' => 'Capsule']));

    $response->assertOk();
    expect($response->json('data.0.name'))->toBe('Capsule');
});

it('resolves a set of type ids to names', function () {
    seedShipReferenceData();
    actingAs(User::factory()->create());

    $response = getJson(route('eve.ship-search', ['kind' => 'type', 'ids' => [638, 16229]]));

    $response->assertOk();
    $names = collect($response->json('data'))->pluck('name');

    expect($names)->toContain('Raven')->toContain('Rokh');
});

it('returns nothing when neither a query nor ids are given', function () {
    seedShipReferenceData();
    actingAs(User::factory()->create());

    getJson(route('eve.ship-search', ['kind' => 'type']))
        ->assertOk()
        ->assertExactJson(['data' => []]);
});

it('validates the kind parameter', function () {
    actingAs(User::factory()->create());

    getJson(route('eve.ship-search', ['q' => 'Raven']))->assertInvalid('kind');
    getJson(route('eve.ship-search', ['kind' => 'wallet', 'q' => 'Raven']))->assertInvalid('kind');
});

it('requires authentication', function () {
    getJson(route('eve.ship-search', ['kind' => 'type', 'q' => 'Raven']))->assertUnauthorized();
});

it('narrows a type search to the given category', function () {
    seedShipReferenceData();
    actingAs(User::factory()->create());

    $response = getJson(route('eve.ship-search', ['kind' => 'type', 'q' => 'Raven', 'category_id' => 6]));

    $response->assertOk();
    $ids = collect($response->json('data'))->pluck('id');

    // Only the ship matches; the blueprint sits in another category.
    expect($ids)->toContain(638)
        ->and($ids)->not->toContain(700);
});

it('includes the type mass in search results', function () {
    seedShipReferenceData();
    Type::query()->whereKey(638)->update(['mass' => 99_300_000]);
    actingAs(User::factory()->create());

    $response = getJson(route('eve.ship-search', ['kind' => 'type', 'q' => 'Raven', 'category_id' => 6]));

    expect(collect($response->json('data'))->firstWhere('id', 638)['mass'])->toBe(99_300_000);
});
