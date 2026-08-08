<?php

declare(strict_types=1);

use App\Enums\AliasScheme;
use App\Enums\BookmarkToken;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\User;

use function Pest\Laravel\actingAs;

function bookmarkFormatUser(Map $map, Permission $permission): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

it('defaults a new map to the default bookmark formats', function () {
    $map = Map::factory()->create()->fresh();

    expect($map->bookmark_format_wormhole)->toBe(BookmarkToken::DEFAULT_WORMHOLE)
        ->and($map->bookmark_format_kspace)->toBe(BookmarkToken::DEFAULT_KSPACE)
        ->and($map->bookmark_alias_scheme)->toBe(AliasScheme::Numeric)
        ->and($map->bookmark_format_return)->toBe(BookmarkToken::DEFAULT_RETURN)
        ->and($map->bookmark_ignored_alias)->toBe(BookmarkToken::DEFAULT_IGNORED_ALIAS);
});

it('lets a manager switch the alias scheme to alphabetical', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_alias_scheme' => 'alphabetical'])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    expect($map->fresh()->bookmark_alias_scheme)->toBe(AliasScheme::Alphabetical);
});

it('forbids a member from updating the alias scheme', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_alias_scheme' => 'alphabetical'])
        ->assertForbidden();

    expect($map->fresh()->bookmark_alias_scheme)->toBe(AliasScheme::Numeric);
});

it('forbids a viewer from updating the alias scheme', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Viewer))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_alias_scheme' => 'alphabetical'])
        ->assertForbidden();
});

it('rejects an invalid alias scheme value', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_alias_scheme' => 'roman-numerals'])
        ->assertSessionHasErrors('bookmark_alias_scheme');

    expect($map->fresh()->bookmark_alias_scheme)->toBe(AliasScheme::Numeric);
});

it('exposes the alias scheme to the mapping settings page', function () {
    $map = Map::factory()->create(['bookmark_alias_scheme' => AliasScheme::Alphabetical]);
    Character::factory()->has(MapAccess::factory(['is_owner' => true])->for($map))->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->get("/maps/{$map->slug}/settings/mapping")
        ->assertSuccessful()
        ->assertInertia(fn ($page) => $page
            ->where('map.bookmark_alias_scheme', AliasScheme::Alphabetical->value)
            ->etc()
        );
});

it('lets a manager update the bookmark formats', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", [
            'bookmark_format_wormhole' => '{sig} {class} {alias}',
            'bookmark_format_kspace' => '{name} {region}',
        ])
        ->assertRedirect();

    $map->refresh();

    expect($map->bookmark_format_wormhole)->toBe('{sig} {class} {alias}')
        ->and($map->bookmark_format_kspace)->toBe('{name} {region}');
});

it('forbids a member from updating the bookmark formats', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_wormhole' => '{sig}'])
        ->assertForbidden();

    expect($map->fresh()->bookmark_format_wormhole)->toBe(BookmarkToken::DEFAULT_WORMHOLE);
});

it('forbids a viewer from updating the bookmark formats', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Viewer))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_wormhole' => '{sig}'])
        ->assertForbidden();
});

it('rejects a format containing an unknown token', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_wormhole' => '{alias} {nonsense}'])
        ->assertSessionHasErrors('bookmark_format_wormhole');

    expect($map->fresh()->bookmark_format_wormhole)->toBe(BookmarkToken::DEFAULT_WORMHOLE);
});

it('accepts the connection tokens', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_wormhole' => '{alias} {occupier} {wh} {size} {mass} {life}'])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    expect($map->fresh()->bookmark_format_wormhole)->toBe('{alias} {occupier} {wh} {size} {mass} {life}');
});

it('accepts a format with literal text around known tokens', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_wormhole' => '[{alias}] {sig}'])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    expect($map->fresh()->bookmark_format_wormhole)->toBe('[{alias}] {sig}');
});

it('lets a manager update the return format and ignored alias', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", [
            'bookmark_format_return' => '*{sig} {class} {alias}',
            'bookmark_ignored_alias' => 'STAGING',
        ])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    $map->refresh();

    expect($map->bookmark_format_return)->toBe('*{sig} {class} {alias}')
        ->and($map->bookmark_ignored_alias)->toBe('STAGING');
});

it('forbids a member from updating the return format and ignored alias', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Member))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_return' => '*{alias}', 'bookmark_ignored_alias' => 'STAGING'])
        ->assertForbidden();

    $map->refresh();

    expect($map->bookmark_format_return)->toBe(BookmarkToken::DEFAULT_RETURN)
        ->and($map->bookmark_ignored_alias)->toBe(BookmarkToken::DEFAULT_IGNORED_ALIAS);
});

it('forbids a viewer from updating the return format and ignored alias', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Viewer))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_return' => '*{alias}', 'bookmark_ignored_alias' => 'STAGING'])
        ->assertForbidden();
});

it('rejects a return format containing an unknown token', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_format_return' => '*{alias} {nonsense}'])
        ->assertSessionHasErrors('bookmark_format_return');

    expect($map->fresh()->bookmark_format_return)->toBe(BookmarkToken::DEFAULT_RETURN);
});

it('accepts an empty ignored alias, which disables the feature', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_ignored_alias' => ''])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    expect($map->fresh()->bookmark_ignored_alias)->toBe('');
});

it('rejects a non-string ignored alias', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_ignored_alias' => ['HOME']])
        ->assertSessionHasErrors('bookmark_ignored_alias');

    expect($map->fresh()->bookmark_ignored_alias)->toBe(BookmarkToken::DEFAULT_IGNORED_ALIAS);
});

it('accepts a plain string ignored alias', function () {
    $map = Map::factory()->create();

    actingAs(bookmarkFormatUser($map, Permission::Manager))
        ->put("/maps/{$map->slug}/bookmark-format", ['bookmark_ignored_alias' => 'STAGING'])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    expect($map->fresh()->bookmark_ignored_alias)->toBe('STAGING');
});
