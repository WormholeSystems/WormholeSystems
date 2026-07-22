<?php

declare(strict_types=1);

use App\Actions\Discord\AutocompleteDiscordAlertsAction;
use App\Actions\Discord\CalculateDiscordRouteAction;
use App\Actions\Discord\CreateDiscordAccountLinkAction;
use App\Actions\Discord\CreateDiscordAlertAction;
use App\Actions\Discord\DeleteDiscordAlertAction;
use App\Actions\Discord\DisableDiscordAlertAction;
use App\Actions\Discord\EnableDiscordAlertAction;
use App\Actions\Discord\GetDiscordAccountStatusAction;
use App\Actions\Discord\ListDiscordMapAlertsAction;
use App\Actions\Discord\ListMyDiscordAlertsAction;
use App\Enums\JumpShipType;
use App\Enums\MapAlertDeliveryType;
use App\Enums\MapAlertDisabledReason;
use App\Enums\MapAlertEventAction;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
use App\Enums\Permission;
use App\Models\Character;
use App\Models\DiscordAccount;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapAlert;
use App\Models\MapWebhook;
use App\Models\Solarsystem;
use App\Models\User;
use Illuminate\Support\Facades\Cache;

function discordAccountWithMapAccess(Map $map, Permission $permission = Permission::Member): DiscordAccount
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    return DiscordAccount::factory()->for($user)->create()->load('user.characters');
}

function discordAlertTarget(int $id, string $name = 'Jita'): Solarsystem
{
    makeSolarsystem($id);
    $system = Solarsystem::query()->findOrFail($id);
    $system->update(['name' => $name]);

    return $system;
}

function discordDmAlert(DiscordAccount $account, Map $map, Solarsystem $target, array $attributes = []): MapAlert
{
    return MapAlert::factory()->discordDm()->create(array_merge([
        'map_id' => $map->id,
        'created_by_user_id' => $account->user_id,
        'target_solarsystem_id' => $target->id,
    ], $attributes));
}

it('reports account status and creates expiring account links', function () {
    $getStatus = app(GetDiscordAccountStatusAction::class);
    $createLink = app(CreateDiscordAccountLinkAction::class);
    $identity = [
        'id' => '123456789012345678',
        'username' => 'mapper',
        'global_name' => 'Mapper',
        'avatar' => null,
    ];

    expect($getStatus->handle(null))->toBe('Your Discord account is not linked.');

    $message = $createLink->handle(null, $identity);
    preg_match('/discord\/link\/([^\s]+)/', $message, $matches);

    expect($message)->toStartWith('Link your account within 15 minutes: ')
        ->and($matches)->toHaveCount(2)
        ->and(Cache::get('discord.link.'.$matches[1]))->toBe($identity);

    $account = DiscordAccount::factory()->create();
    expect($getStatus->handle($account->load('user.characters')))->toContain('Linked to Wormhole Systems as **')
        ->and($createLink->handle($account, $identity))->toBe('Your Discord account is already linked.');
});

it('lists empty and populated personal bot alerts', function () {
    $map = Map::factory()->create(['name' => 'Home Chain']);
    $account = discordAccountWithMapAccess($map);
    $action = app(ListMyDiscordAlertsAction::class);

    expect($action->handle($account))->toBe('You have no proximity alerts.');

    $target = discordAlertTarget(30009601, 'Jita');
    $alert = discordDmAlert($account, $map, $target);
    MapAlert::factory()->webhook()->create([
        'map_id' => $map->id,
        'map_webhook_id' => MapWebhook::factory()->for($map),
        'target_solarsystem_id' => $target->id,
    ]);

    expect($action->handle($account))->toBe("`{$alert->id}` Home Chain: Jita within {$alert->max_jumps} jumps via DM");
});

it('lists only shared map alerts for members and includes private alerts for managers', function () {
    $map = Map::factory()->create(['name' => 'Visible Chain']);
    $member = discordAccountWithMapAccess($map);
    $manager = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009602, 'Amarr');
    $private = discordDmAlert($member, $map, $target);
    $shared = MapAlert::factory()->discordChannel('1', '2')->create([
        'map_id' => $map->id,
        'created_by_user_id' => $member->user_id,
        'target_solarsystem_id' => $target->id,
    ]);
    $action = app(ListDiscordMapAlertsAction::class);

    expect($action->handle($member, $map->id))
        ->toContain("`{$shared->id}`")
        ->not->toContain("`{$private->id}`")
        ->and($action->handle($manager, $map->id))
        ->toContain("`{$shared->id}`")
        ->toContain("`{$private->id}`")
        ->toContain(' by ')
        ->and($action->handle($member, 999999))->toBe('That map is unavailable.');

    $unavailableMap = Map::factory()->create();
    expect($action->handle($member, $unavailableMap->id))->toBe('That map is unavailable.');
});

it('creates a transactional DM alert and lifecycle event', function () {
    $map = Map::factory()->create(['name' => 'Home Chain']);
    $account = discordAccountWithMapAccess($map);
    $target = discordAlertTarget(30009603, 'Dodixie');

    $response = app(CreateDiscordAlertAction::class)->handle($account, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, $map->id, $target->id, 4, null, null, false, MapAlertMentionMode::None, null, null, null);

    $alert = MapAlert::query()->sole();
    expect($response)->toBe('Alert created for **Dodixie** within 4 jumps of **Home Chain**.')
        ->and($alert->delivery_type)->toBe(MapAlertDeliveryType::DiscordDm)
        ->and($alert->mention_mode)->toBe(MapAlertMentionMode::None)
        ->and($alert->events()->sole()->action)->toBe(MapAlertEventAction::Created);
});

it('creates channel alerts for every mention mode', function (MapAlertMentionMode $mention, ?string $roleId) {
    $map = Map::factory()->create();
    $account = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009604);

    $response = app(CreateDiscordAlertAction::class)->handle($account, MapAlertType::Proximity, MapAlertDeliveryType::DiscordChannel, $map->id, $target->id, 2, null, null, false, $mention, 'guild', 'channel', $roleId);

    $alert = MapAlert::query()->sole();
    expect($response)->toStartWith('Alert created')
        ->and($alert->mention_mode)->toBe($mention)
        ->and($alert->discord_guild_id)->toBe('guild')
        ->and($alert->discord_channel_id)->toBe('channel')
        ->and($alert->discord_role_id)->toBe($roleId);
})->with([
    'none' => [MapAlertMentionMode::None, null],
    'creator' => [MapAlertMentionMode::Creator, null],
    'role' => [MapAlertMentionMode::Role, 'role'],
]);

it('rejects invalid alert creation inputs and unauthorized destinations', function () {
    $map = Map::factory()->create();
    $viewer = discordAccountWithMapAccess($map, Permission::Viewer);
    $manager = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009605);
    $action = app(CreateDiscordAlertAction::class);

    expect($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, 999999, $target->id, 2, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('Invalid map or destination.')
        ->and($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, $map->id, 999999, 2, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('That target system is unavailable.')
        ->and($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, $map->id, $target->id, 0, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('The jump count must be between 1 and 20.')
        ->and($action->handle($viewer, MapAlertType::Proximity, null, $map->id, $target->id, 21, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('Invalid map or destination.')
        ->and($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::Webhook, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('Invalid map or destination.')
        ->and($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, $map->id, $target->id, 2, null, null, false, null, null, null, null))
        ->toBe('Invalid map or destination.')
        ->and($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordChannel, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::None, 'guild', 'channel', null))
        ->toBe('Only map managers can create channel alerts.')
        ->and($action->handle($manager, MapAlertType::Proximity, MapAlertDeliveryType::DiscordChannel, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('Channel alerts must be created inside a server channel.')
        ->and($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::Creator, null, null, null))
        ->toBe('Mention options can only be used for channel alerts.')
        ->and($action->handle($manager, MapAlertType::Proximity, MapAlertDeliveryType::DiscordChannel, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::Role, 'guild', 'channel', null))
        ->toBe('Select a role only when the mention option is A role.')
        ->and($action->handle($manager, MapAlertType::Proximity, MapAlertDeliveryType::DiscordChannel, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::None, 'guild', 'channel', 'role'))
        ->toBe('Select a role only when the mention option is A role.')
        ->and(MapAlert::query()->count())->toBe(0);

    $map->mapAccessors()->delete();
    expect($action->handle($viewer, MapAlertType::Proximity, MapAlertDeliveryType::DiscordDm, $map->id, $target->id, 2, null, null, false, MapAlertMentionMode::None, null, null, null))
        ->toBe('You no longer have access to that map.');
});

it('lets a DM creator disable enable and remove their alert through policy and lifecycle', function () {
    $map = Map::factory()->create();
    $account = discordAccountWithMapAccess($map);
    $target = discordAlertTarget(30009606);
    $alert = discordDmAlert($account, $map, $target);
    $disableAlert = app(DisableDiscordAlertAction::class);
    $enableAlert = app(EnableDiscordAlertAction::class);
    $deleteAlert = app(DeleteDiscordAlertAction::class);

    expect($disableAlert->handle($account, $alert->id))->toBe('Alert disabled.')
        ->and($alert->refresh()->is_active)->toBeFalse()
        ->and($enableAlert->handle($account, $alert->id))->toBe('Alert enabled.')
        ->and($alert->refresh()->is_active)->toBeTrue()
        ->and($deleteAlert->handle($account, $alert->id))->toBe('Alert removed.')
        ->and(MapAlert::query()->find($alert->id))->toBeNull();
});

it('requires a current manager for shared alerts and rejects former creators', function () {
    $map = Map::factory()->create();
    $formerCreator = discordAccountWithMapAccess($map);
    $manager = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009607);
    $alert = MapAlert::factory()->discordChannel()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $formerCreator->user_id,
        'target_solarsystem_id' => $target->id,
    ]);
    $disableAlert = app(DisableDiscordAlertAction::class);
    $enableAlert = app(EnableDiscordAlertAction::class);
    $deleteAlert = app(DeleteDiscordAlertAction::class);

    expect($disableAlert->handle($formerCreator, $alert->id))->toBe('That alert does not exist.')
        ->and($deleteAlert->handle($formerCreator, $alert->id))->toBe('That alert does not exist.')
        ->and($disableAlert->handle($manager, $alert->id))->toBe('Alert disabled.')
        ->and($enableAlert->handle($manager, $alert->id))->toBe('Alert enabled.')
        ->and($deleteAlert->handle($manager, $alert->id))->toBe('Alert removed.');
});

it('does not let a former DM creator mutate an alert they cannot view', function () {
    $map = Map::factory()->create();
    $creator = DiscordAccount::factory()->create();
    $target = discordAlertTarget(30009615);
    $alert = discordDmAlert($creator, $map, $target);
    $disableAlert = app(DisableDiscordAlertAction::class);
    $enableAlert = app(EnableDiscordAlertAction::class);
    $deleteAlert = app(DeleteDiscordAlertAction::class);

    expect($disableAlert->handle($creator->load('user.characters'), $alert->id))->toBe('That alert does not exist.')
        ->and($enableAlert->handle($creator, $alert->id))->toBe('That alert does not exist.')
        ->and($deleteAlert->handle($creator, $alert->id))->toBe('That alert does not exist.')
        ->and($alert->refresh()->is_active)->toBeTrue();
});

it('blocks a DM creator from overriding a manager disable', function () {
    $map = Map::factory()->create();
    $creator = discordAccountWithMapAccess($map);
    $manager = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009608);
    $alert = discordDmAlert($creator, $map, $target, [
        'is_active' => false,
        'disabled_at' => now(),
        'disabled_reason' => MapAlertDisabledReason::Manual,
        'disabled_by_user_id' => $manager->user_id,
    ]);

    expect(app(EnableDiscordAlertAction::class)->handle($creator, $alert->id))
        ->toBe('A map manager disabled this alert. A manager must re-enable it.')
        ->and($alert->refresh()->is_active)->toBeFalse();
});

it('validates creator account and map access prerequisites when enabling', function () {
    $map = Map::factory()->create();
    $creator = discordAccountWithMapAccess($map);
    $manager = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009609);
    $dm = discordDmAlert($creator, $map, $target, ['is_active' => false]);
    $channel = MapAlert::factory()->discordChannel()->mentionsCreator()->inactive()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $creator->user_id,
        'target_solarsystem_id' => $target->id,
    ]);
    $action = app(EnableDiscordAlertAction::class);

    $creator->user->characters->first()->mapAccesses()->delete();
    expect($action->handle($manager, $dm->id))->toBe('The alert creator needs map access before this alert can be enabled.');

    MapAccess::factory(['permission' => Permission::Member])->for($map)->for($creator->user->characters->first(), 'accessible')->create();
    $creator->delete();
    expect($action->handle($manager, $dm->id))->toBe('The alert creator must link Discord before this alert can be enabled.')
        ->and($action->handle($manager, $channel->id))->toBe('The alert creator must link Discord before this alert can be enabled.');
});

it('autocompletes only authorized bot alerts matching the query with a 25 choice limit', function () {
    $map = Map::factory()->create(['name' => 'Alpha Chain']);
    $otherMap = Map::factory()->create(['name' => 'Hidden Chain']);
    $account = discordAccountWithMapAccess($map, Permission::Manager);
    $target = discordAlertTarget(30009610, 'Needle Target');
    $hiddenTarget = discordAlertTarget(30009611, 'Needle Hidden');
    $bulkTarget = discordAlertTarget(30009614, 'Bulk Target');
    $matching = MapAlert::factory()->discordChannel()->create(['map_id' => $map->id, 'target_solarsystem_id' => $target->id]);
    MapAlert::factory()->discordChannel()->create(['map_id' => $otherMap->id, 'target_solarsystem_id' => $hiddenTarget->id]);
    MapAlert::factory()->webhook()->create([
        'map_id' => $map->id,
        'map_webhook_id' => MapWebhook::factory()->for($map),
        'target_solarsystem_id' => $target->id,
    ]);
    MapAlert::factory()->count(30)->discordChannel()->create(['map_id' => $map->id, 'target_solarsystem_id' => $bulkTarget->id]);
    $action = app(AutocompleteDiscordAlertsAction::class);

    $filtered = $action->handle($account, 'Needle Target');
    expect($filtered)->toHaveCount(1)
        ->and($filtered[0]['value'])->toBe((string) $matching->id)
        ->and($action->handle($account, ''))->toHaveCount(25);
});

it('calculates routes and reports unavailable maps and missing routes', function () {
    $map = Map::factory()->create();
    $account = discordAccountWithMapAccess($map);
    $origin = placeMapSolarsystem($map, 30009612);
    Solarsystem::query()->whereKey($origin->solarsystem_id)->update(['name' => 'Origin']);
    $target = discordAlertTarget(30009613, 'Target');
    $action = app(CalculateDiscordRouteAction::class);

    expect($action->handle($account, $map->id, $origin->solarsystem_id))->toBe('**0 jumps**: Origin')
        ->and($action->handle($account, 999999, $target->id))->toBe('That map or target system is unavailable.')
        ->and($action->handle($account, $map->id, $target->id))->toBe('No route was found within 100 jumps.');

    placeMapSolarsystem($map, $target->id, 200, 200);

    expect($action->handle($account, $map->id, $target->id))->toBe('**0 jumps**: Target');
});

it('autocompletes a member creator own direct message alert', function () {
    $map = Map::factory()->create(['name' => 'Member Chain']);
    $account = discordAccountWithMapAccess($map, Permission::Member);
    $target = discordAlertTarget(30009615, 'Member Target');
    $alert = MapAlert::factory()->discordDm()->create([
        'map_id' => $map->id,
        'created_by_user_id' => $account->user_id,
        'target_solarsystem_id' => $target->id,
    ]);

    $choices = app(AutocompleteDiscordAlertsAction::class)->handle($account, 'Member Target');

    expect($choices)->toHaveCount(1)
        ->and($choices[0]['value'])->toBe((string) $alert->id);
});

it('creates a jump range alert delivered by direct message', function () {
    $map = Map::factory()->create(['name' => 'Cap Chain']);
    $account = discordAccountWithMapAccess($map);
    $target = discordAlertTarget(30009616, 'Staging');

    $response = app(CreateDiscordAlertAction::class)->handle(
        $account, MapAlertType::JumpRange, MapAlertDeliveryType::DiscordDm, $map->id, $target->id,
        null, JumpShipType::Carrier, 4, true, MapAlertMentionMode::None, null, null, null,
    );

    $alert = MapAlert::query()->sole();
    expect($response)->toStartWith('Alert created for exits within')
        ->and($alert->type)->toBe(MapAlertType::JumpRange)
        ->and($alert->ship_type)->toBe(JumpShipType::Carrier)
        ->and($alert->jdc_level)->toBe(4)
        ->and($alert->include_highsec)->toBeTrue()
        ->and($alert->max_jumps)->toBeNull();
});

it('rejects a jump range alert without a ship class', function () {
    $map = Map::factory()->create();
    $account = discordAccountWithMapAccess($map);
    $target = discordAlertTarget(30009617);

    $response = app(CreateDiscordAlertAction::class)->handle(
        $account, MapAlertType::JumpRange, MapAlertDeliveryType::DiscordDm, $map->id, $target->id,
        null, null, 4, false, MapAlertMentionMode::None, null, null, null,
    );

    expect($response)->toBe('Jump-range alerts need a ship class and a calibration level between 1 and 5.')
        ->and(MapAlert::query()->count())->toBe(0);
});

it('creates a killmail alert without a target system', function () {
    $map = Map::factory()->create(['name' => 'Kill Chain']);
    $account = discordAccountWithMapAccess($map, Permission::Manager);

    $response = app(CreateDiscordAlertAction::class)->handle(
        $account, MapAlertType::Killmail, MapAlertDeliveryType::DiscordChannel, $map->id, null,
        6, null, null, false, MapAlertMentionMode::Everyone, 'guild', 'channel', null,
    );

    $alert = MapAlert::query()->sole();
    expect($response)->toBe('Alert created for kills within 6 jumps of the **Kill Chain** chain.')
        ->and($alert->type)->toBe(MapAlertType::Killmail)
        ->and($alert->target_solarsystem_id)->toBeNull()
        ->and($alert->max_jumps)->toBe(6)
        ->and($alert->mention_mode)->toBe(MapAlertMentionMode::Everyone);
});
