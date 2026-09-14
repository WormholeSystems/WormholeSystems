<?php

declare(strict_types=1);

namespace App\Actions\MapTransfer;

use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapConnection;
use App\Models\MapRouteSolarsystem;
use App\Models\MapSolarsystemDetails;
use App\Models\Signature;
use Carbon\CarbonImmutable;
use Illuminate\Support\Str;

final readonly class ExportMapAction
{
    public const string FORMAT = 'wormholesystems-map-export';

    public const int VERSION = 1;

    /**
     * Build the export payload for the given sections. Secrets (share token,
     * webhooks, per-user settings) and the owner access row never leave the map.
     *
     * @param  list<string>  $sections
     * @return array<string, mixed>
     */
    public function handle(Map $map, array $sections): array
    {
        $payload = [
            'format' => self::FORMAT,
            'version' => self::VERSION,
            'exported_at' => now()->toIso8601String(),
            'map_name' => $map->name,
            'sections' => [],
        ];

        if (in_array('settings', $sections, true)) {
            $payload['sections']['settings'] = $this->settings($map);
        }

        if (in_array('access', $sections, true)) {
            $payload['sections']['access'] = $this->access($map);
        }

        if (in_array('solarsystems', $sections, true)) {
            $payload['sections']['solarsystems'] = $this->solarsystems($map);
        }

        $connection_indexes = [];

        if (in_array('connections', $sections, true)) {
            [$connections, $connection_indexes] = $this->connections($map);
            $payload['sections']['connections'] = $connections;
        }

        if (in_array('signatures', $sections, true)) {
            $payload['sections']['signatures'] = $this->signatures($map, $connection_indexes);
        }

        if (in_array('routes', $sections, true)) {
            $payload['sections']['routes'] = $this->routes($map);
        }

        return $payload;
    }

    /**
     * @return array<string, mixed>
     */
    private function settings(Map $map): array
    {
        return [
            'name' => $map->name,
            'layout' => $map->layout->value,
            'allow_layout_override' => $map->allow_layout_override,
            'constant_width_enabled' => $map->constant_width_enabled,
            'bookmark_format_wormhole' => $map->bookmark_format_wormhole,
            'bookmark_format_kspace' => $map->bookmark_format_kspace,
            'bookmark_format_return' => $map->bookmark_format_return,
            'bookmark_alias_scheme' => $map->bookmark_alias_scheme->value,
            'bookmark_ignored_alias' => $map->bookmark_ignored_alias,
            'home_solarsystem_id' => $map->home_solarsystem_id,
            'rally_solarsystem_id' => $map->rally_solarsystem_id,
        ];
    }

    /**
     * @return list<array<string, mixed>>
     */
    private function access(Map $map): array
    {
        return $map->mapAccessors()
            ->where('is_owner', false)
            ->with('accessible')
            ->get()
            ->map(fn (MapAccess $access): array => [
                'entity_type' => Str::of(class_basename($access->accessible_type))->lower()->value(),
                'entity_id' => $access->accessible_id,
                'entity_name' => $access->accessible->name,
                'permission' => $access->permission->value,
                'expires_at' => $access->expires_at?->toIso8601String(),
            ])
            ->all();
    }

    /**
     * Details are the superset: every placed system has a details row, but intel
     * can outlive its placement. Details-only rows export with null positions.
     *
     * @return list<array<string, mixed>>
     */
    private function solarsystems(Map $map): array
    {
        return $map->mapSolarsystemDetails()
            ->with('mapSolarsystem')
            ->get()
            ->map(fn (MapSolarsystemDetails $details): array => [
                'solarsystem_id' => $details->solarsystem_id,
                'alias' => $details->mapSolarsystem?->alias,
                'position_x' => $details->mapSolarsystem?->position_x,
                'position_y' => $details->mapSolarsystem?->position_y,
                'pinned' => $details->mapSolarsystem !== null ? (bool) $details->mapSolarsystem->pinned : null,
                'status' => $details->status->value,
                'occupier_alias' => $details->occupier_alias,
                'notes' => $details->notes,
            ])
            ->all();
    }

    /**
     * @return array{0: list<array<string, mixed>>, 1: array<int, int>}
     */
    private function connections(Map $map): array
    {
        $connections = $map->mapConnections()
            ->with(['fromMapSolarsystem:id,solarsystem_id', 'toMapSolarsystem:id,solarsystem_id', 'wormhole:id,name'])
            ->get();

        $indexes = $connections->pluck('id')->flip()->all();

        $exported = $connections
            ->map(fn (MapConnection $connection): array => [
                'from_solarsystem_id' => $connection->fromMapSolarsystem->solarsystem_id,
                'to_solarsystem_id' => $connection->toMapSolarsystem->solarsystem_id,
                'wormhole' => $connection->wormhole?->name,
                'type' => $connection->type->value,
                'mass_status' => $connection->mass_status->value,
                'ship_size' => $connection->ship_size?->value,
                'lifetime' => $connection->lifetime->value,
                'lifetime_updated_at' => CarbonImmutable::make($connection->lifetime_updated_at)?->toIso8601String(),
                'connected_at' => CarbonImmutable::make($connection->connected_at)?->toIso8601String(),
                'preserve_mass' => $connection->preserve_mass,
            ])
            ->all();

        return [$exported, $indexes];
    }

    /**
     * @param  array<int, int>  $connection_indexes
     * @return list<array<string, mixed>>
     */
    private function signatures(Map $map, array $connection_indexes): array
    {
        return Signature::query()
            ->whereIn('map_solarsystem_id', $map->mapSolarsystems()->select('id'))
            ->with(['mapSolarsystem:id,solarsystem_id', 'wormhole:id,name', 'signatureType:id,name', 'signatureCategory:id,code'])
            ->get()
            ->map(fn (Signature $signature): array => [
                'solarsystem_id' => $signature->mapSolarsystem->solarsystem_id,
                'signature_id' => $signature->signature_id,
                'category' => $signature->signatureCategory?->code?->value,
                'type_name' => $signature->signatureType?->name,
                'raw_type_name' => $signature->raw_type_name,
                'wormhole' => $signature->wormhole?->name,
                'connection_index' => $signature->map_connection_id !== null
                    ? ($connection_indexes[$signature->map_connection_id] ?? null)
                    : null,
                'mass_status' => $signature->mass_status?->value,
                'ship_size' => $signature->ship_size?->value,
                'lifetime' => $signature->lifetime->value,
                'lifetime_updated_at' => CarbonImmutable::make($signature->lifetime_updated_at)?->toIso8601String(),
            ])
            ->all();
    }

    /**
     * @return array<string, list<array<string, mixed>>>
     */
    private function routes(Map $map): array
    {
        return [
            'route_solarsystems' => $map->mapRouteSolarsystems()
                ->get()
                ->map(fn (MapRouteSolarsystem $route): array => [
                    'solarsystem_id' => $route->solarsystem_id,
                    'is_pinned' => (bool) $route->is_pinned,
                ])
                ->all(),
            'ignored_solarsystems' => $map->mapIgnoredSolarsystems()
                ->get()
                ->map(fn ($ignored): array => [
                    'solarsystem_id' => $ignored->solarsystem_id,
                ])
                ->all(),
        ];
    }
}
