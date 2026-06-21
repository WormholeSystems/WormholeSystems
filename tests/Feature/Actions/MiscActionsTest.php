<?php

declare(strict_types=1);

use App\Actions\CreateStatisticsAction;
use App\Actions\SetWaypointAction;
use App\Actions\ShipHistories\UpdateShipHistoryAction;
use App\Models\Category;
use App\Models\Character;
use App\Models\Group;
use App\Models\Map;
use App\Models\ShipHistory;
use App\Models\Type;
use Illuminate\Foundation\Console\QueuedCommand;
use Illuminate\Support\Facades\Queue;
use NicolasKion\Esi\DTO\EsiResult;
use NicolasKion\Esi\Esi;

it('asks ESI to set the given destination as a waypoint', function () {
    // Esi makes real authenticated HTTP calls and can't be mocked cleanly, so swap in a
    // double that records the destination it was asked to set.
    $esi = new class extends Esi
    {
        public ?int $destination = null;

        public function __construct() {}

        public function setWaypoint($character, int $destination_id, bool $add_to_beginning = false, bool $clear_other_waypoints = false): EsiResult
        {
            $this->destination = $destination_id;

            throw new RuntimeException('stop');
        }
    };
    $this->instance(Esi::class, $esi);

    try {
        app(SetWaypointAction::class)->handle(Character::factory()->create(), [
            'destination_id' => 30000142,
            'add_to_beginning' => false,
            'clear_other_waypoints' => true,
        ]);
    } catch (RuntimeException) {
        // expected — the double short-circuits before returning an EsiResult
    }

    expect($esi->destination)->toBe(30000142);
});

it('queues statistics generation for a map', function () {
    Queue::fake();
    $map = Map::factory()->create();

    app(CreateStatisticsAction::class)->handle(['map_id' => $map->id]);

    Queue::assertPushed(QueuedCommand::class);
});

it('records a ship in the character ship history', function () {
    $character = Character::factory()->create();
    Category::query()->create(['id' => 6, 'name' => 'Ship']);
    Group::query()->create(['id' => 27, 'name' => 'Battleship', 'category_id' => 6]);
    $type = Type::query()->create(['id' => 17738, 'name' => 'Machariel', 'group_id' => 27]);

    $history = app(UpdateShipHistoryAction::class)->handle(
        character_id: (int) $character->id,
        ship_id: 123456789,
        ship_type_id: (int) $type->id,
        name: 'My Mach',
    );

    expect((int) $history->character_id)->toBe((int) $character->id)
        ->and((int) $history->ship_id)->toBe(123456789)
        ->and(ShipHistory::where('character_id', $character->id)->count())->toBe(1);
});
