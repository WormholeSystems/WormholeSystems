<?php

declare(strict_types=1);

namespace App\Actions\EveScout;

use App\Actions\MapConnections\CreateMapConnectionAction;
use App\Actions\MapSolarsystem\StoreMapSolarsystemAction;
use App\Actions\Signatures\StoreSignatureAction;
use App\Data\EveScoutConnectionData;
use App\Data\NewSignatureData;
use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Enums\ShipSize;
use App\Enums\SignatureCategory;
use App\Events\MapSolarsystems\MapSolarsystemCreatedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\SignatureType;
use App\Models\Solarsystem;
use App\Models\Wormhole;
use App\Services\EveScoutService;
use App\Traits\PositionsMapSolarsystems;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\DB;
use Log;
use Random\RandomException;
use Spatie\LaravelData\Optional;
use Throwable;

final readonly class AddEveScoutConnectionToMapAction
{
    use PositionsMapSolarsystems;

    private const int VERTICAL_SPACING = 60; // 40 + extra spacing

    private const int HORIZONTAL_SPACING = 200;

    private const int MAXIMUM_TRIES = 100;

    public function __construct(
        private EveScoutService $eveScoutService,
        private StoreMapSolarsystemAction $storeMapSolarsystemAction,
        private CreateMapConnectionAction $createMapConnectionAction,
        private StoreSignatureAction $storeSignatureAction,
        #[Config('map.max_size.y')] private int $max_y,
        #[Config('map.max_size.x')] private int $max_x,
        #[Config('map.grid_size')] private int $grid_size,
    ) {}

    /**
     * Add all EVE Scout connections for a special system (Thera/Turnur) to the map
     *
     * Flow:
     * 1. Fetch fresh connections from EVE Scout for the special system
     * 2. Add the special system (Thera/Turnur) to map (if not exists)
     * 3. Add each connected solarsystem to map (if not exists)
     * 4. Add each connection to map (if not exists)
     * 5. Add each signature (if not exist) and match them with connections
     *
     * @throws Throwable
     */
    public function handle(Map $map, Solarsystem $solarsystem): void
    {
        $origin_system_id = $solarsystem->id;
        DB::transaction(function () use ($map, $origin_system_id): void {
            // Step 1: Fetch fresh connections from EVE Scout
            $all_connections = $this->eveScoutService->getWormholeConnections();

            // Filter for connections involving the special system (Thera/Turnur)
            $connections = collect($all_connections)
                ->filter(fn (EveScoutConnectionData $connection): bool => $connection->in_system_id === $origin_system_id
                    || $connection->out_system_id === $origin_system_id);
            if ($connections->isEmpty()) {
                Log::info('No EVE Scout connections found for system ID '.$origin_system_id);

                return;
            }

            $solarsystems = Solarsystem::query()
                ->whereIn('id', $connections->flatMap(fn (EveScoutConnectionData $connection): array => [
                    $connection->in_system_id,
                    $connection->out_system_id,
                ])->unique()->values()->all())
                ->get()
                ->keyBy('id');

            $connections = $connections->sort(fn (EveScoutConnectionData $a, EveScoutConnectionData $b): int => $this->sortConnection($a, $b, $solarsystems, $origin_system_id));

            // Step 2: Add the special system (Thera/Turnur) to map (if not exists)
            ['system' => $specialMapSystem] = $this->addSystemToMap($map, $origin_system_id);

            // Get wormhole signature category for lookups
            $wormholeCategory = \App\Models\SignatureCategory::query()
                ->where('code', SignatureCategory::Wormhole->value)
                ->first();

            // Track the vertical position for sideways tree layout
            $currentYPosition = $specialMapSystem->position_y;

            foreach ($connections as $connection) {
                // Determine which system is the special one and which is the destination
                $isSpecialIn = $connection->in_system_id === $origin_system_id;
                $destinationSystemId = $isSpecialIn ? $connection->out_system_id : $connection->in_system_id;
                $specialSignature = $isSpecialIn ? $connection->in_signature : $connection->out_signature;
                $destinationSignature = $isSpecialIn ? $connection->out_signature : $connection->in_signature;
                $wormholeType = $connection->wormhole_type;

                // Step 3: Add each connected solarsystem to map (if not exists)
                // Position systems to the right of the special system, vertically spaced
                ['system' => $destinationMapSystem, 'was_created' => $wasCreated] = $this->addSystemToMapSideways(
                    $map,
                    $destinationSystemId,
                    $specialMapSystem,
                    $currentYPosition
                );

                if ($wasCreated) {
                    // Move down for the next system (vertical spacing)
                    $currentYPosition += self::VERTICAL_SPACING;
                }

                // Lookup wormhole and signature type
                $wormhole = Wormhole::query()->where('name', $wormholeType)->first();
                $signatureType = null;
                if ($wormhole && $wormholeCategory) {
                    $signatureType = SignatureType::query()
                        ->where('signature_category_id', $wormholeCategory->id)
                        ->whereHas('wormhole', fn ($q) => $q->where('id', $wormhole->id))
                        ->first();
                }

                // Step 4: Add connection to map (if not exists)
                $mapConnection = $this->getOrCreateConnection(
                    $specialMapSystem,
                    $destinationMapSystem,
                    $wormhole?->id
                );

                // Step 5: Add each signature (if not exist) and match them with connections
                $this->createSignatureIfNotExists(
                    $specialMapSystem,
                    $specialSignature,
                    $wormholeCategory?->id,
                    $signatureType?->id,
                    $mapConnection->id
                );

                $this->createSignatureIfNotExists(
                    $destinationMapSystem,
                    $destinationSignature,
                    $wormholeCategory?->id,
                    $signatureType?->id,
                    $mapConnection->id
                );
            }

            broadcast(new MapSolarsystemCreatedEvent($map->id))->toOthers();
        }, attempts: 5);
    }

    /**
     * Sort connections by destination system security status (desc), then class
     *
     * @param  Collection<int, Solarsystem>  $solarsystems
     */
    private function sortConnection(EveScoutConnectionData $a, EveScoutConnectionData $b, Collection $solarsystems, int $origin_system_id): int
    {
        $aDestinationId = $a->in_system_id === $origin_system_id ? $a->out_system_id : $a->in_system_id;
        $bDestinationId = $b->in_system_id === $origin_system_id ? $b->out_system_id : $b->in_system_id;

        $aSystem = $solarsystems->get($aDestinationId);
        $bSystem = $solarsystems->get($bDestinationId);

        $a_value = $aSystem->wormholeSystem?->class ? -$aSystem->wormholeSystem->class : $aSystem->security;
        $b_value = $bSystem->wormholeSystem?->class ? -$bSystem->wormholeSystem->class : $bSystem->security;

        return $b_value <=> $a_value;
    }

    /**
     * Add a system to the map at a smart position
     * If reference system is provided, position relative to it
     * Otherwise use a default position
     *
     * @return array{system: MapSolarsystem, was_created: bool}
     *
     * @throws RandomException
     */
    private function addSystemToMap(Map $map, int $solarsystemId): array
    {
        // Check if system already exists on map (with position)
        $existingSystem = $map->mapSolarsystems()
            ->where('solarsystem_id', $solarsystemId)
            ->isOnMap()
            ->first();

        if ($existingSystem) {
            return ['system' => $existingSystem, 'was_created' => false];
        }

        // Get all occupied positions for collision detection
        $occupiedPositions = $map->mapSolarsystems()
            ->isOnMap()
            ->get(['position_x', 'position_y', 'id']);

        // Try default starting position for special system (Thera/Turnur)
        $position_x = 200;
        $position_y = 100;

        // Check if default position is free
        $testPosition = $this->constrainPositionToMapBounds($position_x, $position_y);
        if ($occupiedPositions->hasElementAtPosition(
            $testPosition['position_x'],
            $testPosition['position_y'],
            self::VERTICAL_SPACING,
            self::HORIZONTAL_SPACING
        )) {
            // Default position is occupied, scan for a free position
            // Scan top to bottom, then left to right
            $found = false;
            for ($x = 100; $x <= $this->max_x - 100; $x += self::HORIZONTAL_SPACING) {
                for ($y = 100; $y <= $this->max_y - 100; $y += self::VERTICAL_SPACING) {
                    $testPosition = $this->constrainPositionToMapBounds($x, $y);
                    if (! $occupiedPositions->hasElementAtPosition(
                        $testPosition['position_x'],
                        $testPosition['position_y'],
                        self::VERTICAL_SPACING,
                        self::HORIZONTAL_SPACING
                    )) {
                        $position_x = $testPosition['position_x'];
                        $position_y = $testPosition['position_y'];
                        $found = true;
                        break 2;
                    }
                }
            }

            // If still no free position found, use the default position anyway
            if (! $found) {
                $position_x = $testPosition['position_x'];
                $position_y = $testPosition['position_y'];
            }
        } else {
            $position_x = $testPosition['position_x'];
            $position_y = $testPosition['position_y'];
        }

        // Add the system to the map
        $system = $this->storeMapSolarsystemAction->handle($map, [
            'solarsystem_id' => $solarsystemId,
            'position_x' => $position_x,
            'position_y' => $position_y,
        ]);

        return ['system' => $system, 'was_created' => true];
    }

    /**
     * Add a system to the map in a sideways tree layout
     * Connected systems appear to the right of the parent, spaced vertically
     *
     * @return array{system: MapSolarsystem, was_created: bool}
     *
     * @throws RandomException
     */
    private function addSystemToMapSideways(
        Map $map,
        int $solarsystemId,
        MapSolarsystem $parentSystem,
        int $yPosition
    ): array {
        // Check if system already exists on map (with position)
        $existingSystem = $map->mapSolarsystems()
            ->where('solarsystem_id', $solarsystemId)
            ->isOnMap()
            ->first();

        if ($existingSystem) {
            return ['system' => $existingSystem, 'was_created' => false];
        }

        // Get all occupied positions for collision detection
        $occupiedPositions = $map->mapSolarsystems()
            ->isOnMap()
            ->get(['position_x', 'position_y', 'id']);

        // Try to find a free position to the right of the parent
        $position_x = $parentSystem->position_x + self::HORIZONTAL_SPACING;
        $position_y = $yPosition;

        // Find first available position (checking for collisions)
        $attempts = 0;
        while ($attempts < self::MAXIMUM_TRIES) {
            // Constrain and snap to grid
            $testPosition = $this->constrainPositionToMapBounds($position_x, $position_y);

            // Check if this position is free
            if (! $occupiedPositions->hasElementAtPosition(
                $testPosition['position_x'],
                $testPosition['position_y'],
                self::VERTICAL_SPACING,
                self::HORIZONTAL_SPACING
            )) {
                // Position is free, use it
                $position_x = $testPosition['position_x'];
                $position_y = $testPosition['position_y'];
                break;
            }

            // Position occupied, try next Y position
            $position_y += self::VERTICAL_SPACING;

            // If we exceed max Y, move to next column
            if ($position_y > $this->max_y) {
                $position_y = $parentSystem->position_y;
                $position_x += self::HORIZONTAL_SPACING;
            }

            $attempts++;
        }

        // If we couldn't find a position, use random position as fallback
        if ($attempts >= self::MAXIMUM_TRIES) {
            [
                'position_x' => $position_x,
                'position_y' => $position_y,
            ] = $this->getRandomPositionAroundSystem($parentSystem, self::HORIZONTAL_SPACING, self::VERTICAL_SPACING);
        }

        // Add the system to the map
        $system = $this->storeMapSolarsystemAction->handle($map, [
            'solarsystem_id' => $solarsystemId,
            'position_x' => $position_x,
            'position_y' => $position_y,
        ]);

        return ['system' => $system, 'was_created' => true];
    }

    /**
     * Get or create a connection between two systems using existing action
     */
    private function getOrCreateConnection(
        MapSolarsystem $specialMapSystem,
        MapSolarsystem $destinationMapSystem,
        ?int $wormholeId
    ): MapConnection {
        // Check if a connection already exists between these systems
        $existingConnection = MapConnection::query()
            ->where('map_id', $specialMapSystem->map_id)
            ->where(function ($query) use ($specialMapSystem, $destinationMapSystem): void {
                $query->where(function ($q) use ($specialMapSystem, $destinationMapSystem): void {
                    $q->where('from_map_solarsystem_id', $specialMapSystem->id)
                        ->where('to_map_solarsystem_id', $destinationMapSystem->id);
                })->orWhere(function ($q) use ($specialMapSystem, $destinationMapSystem): void {
                    $q->where('from_map_solarsystem_id', $destinationMapSystem->id)
                        ->where('to_map_solarsystem_id', $specialMapSystem->id);
                });
            })
            ->first();

        if ($existingConnection) {
            return $existingConnection;
        }

        // Create a new connection using existing action
        return $this->createMapConnectionAction->handle([
            'from_map_solarsystem_id' => $specialMapSystem->id,
            'to_map_solarsystem_id' => $destinationMapSystem->id,
            'wormhole_id' => $wormholeId,
            'mass_status' => MassStatus::Fresh,
            'ship_size' => ShipSize::Large,
            'lifetime' => LifetimeStatus::Healthy,
        ]);
    }

    /**
     * Create a signature if it doesn't exist, using existing StoreSignatureAction
     *
     * @throws Throwable
     */
    private function createSignatureIfNotExists(
        MapSolarsystem $mapSolarsystem,
        string $signatureId,
        ?int $signatureCategoryId,
        ?int $signatureTypeId,
        int $mapConnectionId
    ): void {
        // Check if signature already exists
        $existingSignature = $mapSolarsystem->signatures()
            ->where('signature_id', $signatureId)
            ->first();

        if ($existingSignature) {
            // Update the signature with latest data
            $updateData = [];

            if ($existingSignature->map_connection_id === null) {
                $updateData['map_connection_id'] = $mapConnectionId;
            }

            if ($signatureTypeId !== null && $existingSignature->signature_type_id === null) {
                $updateData['signature_type_id'] = $signatureTypeId;
            }

            if ($signatureCategoryId !== null && $existingSignature->signature_category_id === null) {
                $updateData['signature_category_id'] = $signatureCategoryId;
            }

            if ($updateData !== []) {
                $existingSignature->update($updateData);
            }

            return;
        }

        // Create the signature using existing action
        $this->storeSignatureAction->handle(
            $mapSolarsystem,
            new NewSignatureData(
                signature_id: $signatureId,
                signature_type_id: $signatureTypeId ?? Optional::create(),
                signature_category_id: $signatureCategoryId ?? Optional::create(),
                map_connection_id: $mapConnectionId,
                mass_status: Optional::create(),
                ship_size: Optional::create(),
                lifetime: Optional::create(),
                lifetime_updated_at: Optional::create(),
                raw_type_name: Optional::create(),
            )
        );
    }
}
