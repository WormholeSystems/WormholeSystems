<?php

declare(strict_types=1);

use App\Actions\MapAccess\CreateMapAccessAction;
use App\Actions\MapAccess\DeleteMapAccessAction;
use App\Actions\MapAccess\UpdateMapAccessAction;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;

it('grants a character access to a map', function () {
    $map = Map::factory()->create();
    $character = Character::factory()->create();

    $access = app(CreateMapAccessAction::class)->handle($map, $character, permission: Permission::Member);

    expect($access->map_id)->toBe($map->id)
        ->and($access->accessible_id)->toBe($character->id)
        ->and($access->permission)->toBe(Permission::Member)
        ->and($access->is_owner)->toBeFalse();
});

it('updates a map access permission', function () {
    $map = Map::factory()->create();
    $access = app(CreateMapAccessAction::class)->handle($map, Character::factory()->create(), permission: Permission::Member);

    app(UpdateMapAccessAction::class)->handle($access, permission: Permission::Manager);

    expect($access->fresh()->permission)->toBe(Permission::Manager);
});

it('deletes a map access', function () {
    $map = Map::factory()->create();
    $access = app(CreateMapAccessAction::class)->handle($map, Character::factory()->create());

    app(DeleteMapAccessAction::class)->handle($access);

    expect(MapAccess::find($access->id))->toBeNull();
});
