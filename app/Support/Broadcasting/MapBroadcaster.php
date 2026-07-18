<?php

declare(strict_types=1);

namespace App\Support\Broadcasting;

use App\Events\MapConnections\MapConnectionsRemovedEvent;
use App\Events\MapConnections\MapConnectionsUpsertedEvent;
use App\Events\Maps\MapMetadataUpdatedEvent;
use App\Events\Maps\MapResyncEvent;
use App\Events\MapSolarsystems\MapSolarsystemsRemovedEvent;
use App\Events\MapSolarsystems\MapSolarsystemsUpsertedEvent;
use App\Events\Signatures\SignaturesChangedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Collection;

/**
 * Single seam for the entity-payload map broadcasts. Payload events go to every
 * subscriber (no toOthers()) so the acting client converges on the same state;
 * the legacy contentless pings stay in the actions and keep using toOthers().
 */
final readonly class MapBroadcaster
{
    /**
     * Reverb rejects messages above 10KB (config/reverb.php max_message_size), so
     * changes touching more entities than this are broadcast as a resync ping
     * instead of full entity payloads.
     */
    private const int MAX_ENTITIES_PER_EVENT = 10;

    /**
     * Id-only payloads are tiny, so removals tolerate far more entries before
     * falling back to a resync ping.
     */
    private const int MAX_IDS_PER_EVENT = 40;

    /**
     * @param  Collection<int, MapSolarsystem>  $map_solarsystems
     */
    public function systemsUpserted(int $map_id, Collection $map_solarsystems): void
    {
        if ($map_solarsystems->count() > self::MAX_ENTITIES_PER_EVENT) {
            $this->resync($map_id);

            return;
        }

        broadcast(new MapSolarsystemsUpsertedEvent($map_id, $map_solarsystems));
    }

    /**
     * @param  int[]  $map_solarsystem_ids
     * @param  int[]  $map_connection_ids
     */
    public function systemsRemoved(int $map_id, array $map_solarsystem_ids, array $map_connection_ids): void
    {
        if (count($map_solarsystem_ids) + count($map_connection_ids) > self::MAX_IDS_PER_EVENT) {
            $this->resync($map_id);

            return;
        }

        broadcast(new MapSolarsystemsRemovedEvent($map_id, array_values($map_solarsystem_ids), array_values($map_connection_ids)));
    }

    /**
     * @param  Collection<int, MapConnection>  $map_connections
     */
    public function connectionsUpserted(int $map_id, Collection $map_connections): void
    {
        if ($map_connections->count() > self::MAX_ENTITIES_PER_EVENT) {
            $this->resync($map_id);

            return;
        }

        broadcast(new MapConnectionsUpsertedEvent($map_id, $map_connections));
    }

    /**
     * @param  int[]  $map_connection_ids
     * @param  int[]  $map_solarsystem_ids
     */
    public function connectionsRemoved(int $map_id, array $map_connection_ids, array $map_solarsystem_ids = []): void
    {
        if (count($map_connection_ids) + count($map_solarsystem_ids) > self::MAX_IDS_PER_EVENT) {
            $this->resync($map_id);

            return;
        }

        broadcast(new MapConnectionsRemovedEvent($map_id, array_values($map_connection_ids), array_values($map_solarsystem_ids)));
    }

    /**
     * Broadcast the system's signature counts, reloaded fresh so the payload
     * reflects the state after the mutation. Never includes signature bodies.
     */
    public function signaturesChanged(MapSolarsystem $map_solarsystem): void
    {
        // Deleting signatures can also roll their connections and remove the
        // system in the same transaction. If it is gone there are no counts to
        // broadcast — loadCount would re-query by key, find nothing, and fatal on
        // getAttributes(). The removal itself is announced by its own event.
        $map_solarsystem = $map_solarsystem->fresh();

        if (! $map_solarsystem instanceof MapSolarsystem) {
            return;
        }

        $map_solarsystem->loadCount(['signatures', 'wormholeSignatures', 'uncategorizedSignatures']);

        broadcast(new SignaturesChangedEvent(
            $map_solarsystem->map_id,
            $map_solarsystem->id,
            [
                'signatures_count' => (int) $map_solarsystem->signatures_count,
                'wormhole_signatures_count' => (int) $map_solarsystem->wormhole_signatures_count,
                'uncategorized_signatures_count' => (int) $map_solarsystem->uncategorized_signatures_count,
            ],
        ));
    }

    public function metadataUpdated(Map $map): void
    {
        broadcast(new MapMetadataUpdatedEvent($map));
    }

    public function resync(int $map_id): void
    {
        broadcast(new MapResyncEvent($map_id));
    }
}
