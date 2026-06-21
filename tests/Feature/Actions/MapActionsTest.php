<?php

declare(strict_types=1);

use App\Actions\Map\CreateMapAction;
use App\Actions\Map\UpdateMapAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;

it('creates a map owned by the character', function () {
    $character = Character::factory()->create();

    $map = app(CreateMapAction::class)->handle($character, ['name' => 'Wormhole HQ']);

    expect($map->name)->toBe('Wormhole HQ')
        ->and(MapAccess::query()
            ->where('map_id', $map->id)
            ->where('accessible_id', $character->id)
            ->where('is_owner', true)
            ->exists())->toBeTrue();
});

it('updates a map', function () {
    $map = Map::factory()->create(['name' => 'Old name']);

    app(UpdateMapAction::class)->handle($map, ['name' => 'New name']);

    expect($map->fresh()->name)->toBe('New name');
});
