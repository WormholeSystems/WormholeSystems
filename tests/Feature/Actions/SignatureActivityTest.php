<?php

declare(strict_types=1);

use App\Actions\DeleteSignaturesAction;
use App\Actions\EveScout\AddEveScoutConnectionToMapAction;
use App\Actions\Signatures\DeleteSignatureAction;
use App\Actions\Signatures\PasteSignaturesAction;
use App\Actions\Signatures\StoreSignatureAction;
use App\Actions\Signatures\UpdateSignatureAction;
use App\Actions\Tracking\StoreTrackingAction;
use App\Data\NewSignatureData;
use App\Data\SignatureData;
use App\Data\SignaturesData;
use App\Data\TrackingData;
use App\Enums\Permission;
use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\Signature;
use App\Models\SignatureActivity;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use App\Models\Solarsystem;
use App\Models\User;
use App\Support\ActingCharacter;
use Carbon\CarbonImmutable;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Session;

use function Pest\Laravel\actingAs;

function signatureActivityUser(Map $map, Permission $permission = Permission::Member): User
{
    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => $permission])->for($map)))
        ->create();

    $user->forceFill(['preferred_character_id' => $user->characters()->value('id')])->save();

    return $user->refresh();
}

it('records exactly one created row when storing a signature through the controller', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013001);
    $user = signatureActivityUser($map);
    $character = $user->characters()->firstOrFail();

    actingAs($user)
        ->post("/map-solarsystems/{$system->id}/signatures", ['signature_id' => 'ABC-123'])
        ->assertRedirect();

    $signature = Signature::query()->where('signature_id', 'ABC-123')->firstOrFail();

    $activity = SignatureActivity::sole();
    expect($activity->action)->toBe(SignatureActivityAction::Created)
        ->and($activity->map_id)->toBe($map->id)
        ->and($activity->character_id)->toBe($character->id)
        ->and($activity->signature_id)->toBe($signature->id)
        ->and($activity->solarsystem_id)->toBe(30013001);
});

it('records exactly one updated row when updating a signature through the controller', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013002);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);
    $user = signatureActivityUser($map);
    $character = $user->characters()->firstOrFail();

    actingAs($user)
        ->put("/signatures/{$signature->id}", ['signature_id' => 'XYZ-999'])
        ->assertRedirect();

    $activity = SignatureActivity::sole();
    expect($activity->action)->toBe(SignatureActivityAction::Updated)
        ->and($activity->character_id)->toBe($character->id)
        ->and($activity->signature_id)->toBe($signature->id);
});

it('records exactly one deleted row that survives the signature being destroyed through the controller', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013003);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);
    $signature_id = $signature->id;
    $user = signatureActivityUser($map);
    $character = $user->characters()->firstOrFail();

    actingAs($user)
        ->delete("/signatures/{$signature->id}")
        ->assertRedirect();

    expect(Signature::query()->find($signature_id))->toBeNull();

    $activity = SignatureActivity::sole();
    expect($activity->action)->toBe(SignatureActivityAction::Deleted)
        ->and($activity->character_id)->toBe($character->id)
        ->and($activity->signature_id)->toBe($signature_id)
        ->and($activity->solarsystem_id)->toBe(30013003);
});

it('dedupes two updates by the same character on the same day into one row', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013004);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);
    $actor = Character::factory()->create();
    $category_id = SignatureCategory::query()->value('id');

    app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['signature_category_id' => $category_id]), actor: $actor);
    app(UpdateSignatureAction::class)->handle($signature->fresh(), SignatureData::from(['raw_type_name' => 'Some Event Site']), actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(1);
});

it('records separate rows for two different characters updating the same signature on the same day', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013005);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);
    $actor_a = Character::factory()->create();
    $actor_b = Character::factory()->create();

    app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['raw_type_name' => 'Event A']), actor: $actor_a);
    app(UpdateSignatureAction::class)->handle($signature->fresh(), SignatureData::from(['raw_type_name' => 'Event B']), actor: $actor_b);

    expect(SignatureActivity::query()->count())->toBe(2);
});

it('cancels a create-then-delete churn within 60 seconds, recording nothing', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013006);
    $actor = Character::factory()->create();

    $signature = app(StoreSignatureAction::class)->handle($system, NewSignatureData::from(['signature_id' => 'ABC-123']), actor: $actor);
    app(DeleteSignatureAction::class)->handle($signature, actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(0);
});

it('still fires the churn guard when the signature code is set between create and delete', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013007);
    $actor = Character::factory()->create();

    $signature = app(StoreSignatureAction::class)->handle($system, NewSignatureData::from([]), actor: $actor);
    $signature = app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['signature_id' => 'ABC-123']), actor: $actor);
    app(DeleteSignatureAction::class)->handle($signature, actor: $actor);

    $activity = SignatureActivity::sole();
    expect($activity->action)->toBe(SignatureActivityAction::Updated);
});

it('does not cancel churn once more than 60 seconds have passed, and pins created_at to the app clock', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013008);
    $actor = Character::factory()->create();

    $frozen = CarbonImmutable::parse('2026-06-15 12:00:00', 'UTC');
    $this->travelTo($frozen);

    $signature = app(StoreSignatureAction::class)->handle($system, NewSignatureData::from(['signature_id' => 'ABC-123']), actor: $actor);

    $this->travel(90)->seconds();

    app(DeleteSignatureAction::class)->handle($signature, actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(2);

    $created = SignatureActivity::query()->where('action', SignatureActivityAction::Created->value)->sole();
    expect($created->created_at->equalTo($frozen))->toBeTrue();

    SignatureActivity::query()->where('action', SignatureActivityAction::Deleted->value)->sole();
});

it('buckets activity_date and created_at on the same day for a write at 23:59:59Z', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013009);
    $actor = Character::factory()->create();

    $this->travelTo(CarbonImmutable::parse('2026-06-30 23:59:59', 'UTC'));

    app(StoreSignatureAction::class)->handle($system, NewSignatureData::from(['signature_id' => 'ABC-123']), actor: $actor);

    $activity = SignatureActivity::sole();
    expect($activity->activity_date->toDateString())->toBe('2026-06-30')
        ->and($activity->created_at->toDateString())->toBe('2026-06-30');
});

it('records one created row per new signature when pasting', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013010);
    $actor = Character::factory()->create();

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111'],
            ['signature_id' => 'BBB-222'],
            ['signature_id' => 'CCC-333'],
        ],
    ]), actor: $actor);

    expect(SignatureActivity::query()->where('action', SignatureActivityAction::Created->value)->count())->toBe(3);
});

it('records nothing for anomalies pasted alongside signatures, including a combat anomaly', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013011);
    $actor = Character::factory()->create();
    $combat_category_id = SignatureCategory::query()->where('code', 'combat')->value('id');

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'is_anomaly' => true],
            ['signature_id' => 'BBB-222', 'is_anomaly' => true, 'signature_category_id' => $combat_category_id],
            ['signature_id' => 'CCC-333', 'is_anomaly' => true],
            ['signature_id' => 'DDD-444', 'is_anomaly' => false],
            ['signature_id' => 'EEE-555', 'is_anomaly' => false],
        ],
    ]), actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(2)
        ->and(SignatureActivity::query()->where('action', SignatureActivityAction::Created->value)->count())->toBe(2);
});

it('records nothing when updating or deleting an anomaly signature', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013012);
    $actor = Character::factory()->create();

    $signature = $system->signatures()->create(['signature_id' => 'AAA-111', 'is_anomaly' => true]);

    app(UpdateSignatureAction::class)->handle($signature, SignatureData::from(['raw_type_name' => 'Some Ore Site']), actor: $actor);
    app(DeleteSignatureAction::class)->handle($signature->fresh(), actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(0);
});

it('does not score a point when a paste omitting is_anomaly re-pastes an existing anomaly', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013021);
    $actor = Character::factory()->create();

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [['signature_id' => 'AAA-111', 'is_anomaly' => true]],
    ]), actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(0);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [['signature_id' => 'AAA-111']],
    ]), actor: $actor);

    expect(SignatureActivity::query()->count())->toBe(0);
});

it('does not record an updated row when re-pasting an unchanged scan', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013013);
    $actor = Character::factory()->create();

    $payload = [
        'map_solarsystem_id' => $system->id,
        'signatures' => [['signature_id' => 'AAA-111']],
    ];

    app(PasteSignaturesAction::class)->handle(SignaturesData::from($payload), actor: $actor);

    expect(SignatureActivity::query()->where('action', SignatureActivityAction::Created->value)->count())->toBe(1);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from($payload), actor: $actor);

    expect(SignatureActivity::query()->where('action', SignatureActivityAction::Updated->value)->count())->toBe(0);
});

it('records exactly one updated row when a pasted signature type changes', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013014);
    $actor = Character::factory()->create();

    $data_category_id = SignatureCategory::query()->where('code', 'data')->value('id');
    $type_ids = SignatureType::query()->where('signature_category_id', $data_category_id)->limit(2)->pluck('id')->values();

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'signature_category_id' => $data_category_id, 'signature_type_id' => $type_ids[0]],
            ['signature_id' => 'BBB-222', 'signature_category_id' => $data_category_id, 'signature_type_id' => $type_ids[0]],
        ],
    ]), actor: $actor);

    expect(SignatureActivity::query()->where('action', SignatureActivityAction::Created->value)->count())->toBe(2);

    app(PasteSignaturesAction::class)->handle(SignaturesData::from([
        'map_solarsystem_id' => $system->id,
        'signatures' => [
            ['signature_id' => 'AAA-111', 'signature_category_id' => $data_category_id, 'signature_type_id' => $type_ids[1]],
            ['signature_id' => 'BBB-222', 'signature_category_id' => $data_category_id, 'signature_type_id' => $type_ids[0]],
        ],
    ]), actor: $actor);

    expect(SignatureActivity::query()->where('action', SignatureActivityAction::Updated->value)->count())->toBe(1);
});

it('records one deleted row per signature in a bulk delete', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013015);
    $actor = Character::factory()->create();

    $ids = collect(range(1, 5))
        ->map(fn (int $i): int => $system->signatures()->create(['signature_id' => sprintf('AAA-%03d', $i)])->id)
        ->all();

    app(DeleteSignaturesAction::class)->handle($system, $ids, actor: $actor);

    expect(SignatureActivity::query()->where('action', SignatureActivityAction::Deleted->value)->count())->toBe(5);
});

it('records nothing for signatures created via the EVE-Scout import', function () {
    $thera_id = makeSolarsystem(31000006);
    $destination_id = makeSolarsystem(30013016);
    $map = Map::factory()->create();

    Http::fake(['*' => Http::response([[
        'in_system_id' => $thera_id,
        'out_system_id' => $destination_id,
        'in_signature' => 'AAA-111',
        'out_signature' => 'BBB-222',
        'wh_type' => 'K162',
        'life' => 'stable',
        'mass' => 'stable',
        'remaining_hours' => 16,
        'completed' => true,
        'signature_type' => 'wormhole',
        'created_at' => null,
    ]])]);

    app(AddEveScoutConnectionToMapAction::class)->handle($map, Solarsystem::find($thera_id));

    expect(SignatureActivity::query()->count())->toBe(0);
});

it('records nothing when tracking a jump via StoreTrackingAction', function () {
    $map = Map::factory()->create();
    $origin = placeMapSolarsystem($map, 30013017);
    $target_id = makeSolarsystem(30013018);

    app(StoreTrackingAction::class)->handle(TrackingData::from([
        'from_map_solarsystem_id' => $origin->id,
        'to_solarsystem_id' => $target_id,
    ]));

    expect(SignatureActivity::query()->count())->toBe(0);
});

it('records nothing when the scheduled command deletes old signatures', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013019);
    $signature = $system->signatures()->create(['signature_id' => 'ABC-123']);
    $signature->forceFill(['created_at' => now()->subDays(10)])->save();

    Artisan::call('app:delete-old-signatures');

    expect(Signature::query()->find($signature->id))->toBeNull()
        ->and(SignatureActivity::query()->count())->toBe(0);
});

it('resolves null for a user with zero characters, without throwing or touching the session', function () {
    $user = User::factory()->create();

    expect(ActingCharacter::resolve($user))->toBeNull()
        ->and(ActingCharacter::resolve(null))->toBeNull()
        ->and(Session::get(User::SESSION_ACTIVE_CHARACTER_ID))->toBeNull();
});

it('falls back to the owned character, without clearing a dangling preferred_character_id, when writing a signature', function () {
    $map = Map::factory()->create();
    $system = placeMapSolarsystem($map, 30013020);

    $user = User::factory()
        ->has(Character::factory()->has(MapAccess::factory(['permission' => Permission::Member])->for($map)))
        ->create();
    $owned = $user->characters()->firstOrFail();

    $foreign = Character::factory()->create();
    $user->forceFill(['preferred_character_id' => $foreign->id])->save();

    actingAs($user->refresh())
        ->post("/map-solarsystems/{$system->id}/signatures", ['signature_id' => 'ABC-123'])
        ->assertRedirect()
        ->assertSessionHasNoErrors();

    $activity = SignatureActivity::sole();
    expect($activity->character_id)->toBe($owned->id)
        ->and((int) $user->fresh()->preferred_character_id)->toBe((int) $foreign->id)
        ->and(Session::get(User::SESSION_ACTIVE_CHARACTER_ID))->toBeNull();
});
