<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\MapAlert;
use App\Models\MapSolarsystem;
use App\Services\Routing\ProximityResult;

final class PersonalProximityAlertEmbed
{
    use ColorsBySecurity;
    use ResolvesSolarsystems;

    /** @return array<string, mixed> */
    public function build(MapAlert $alert, MapSolarsystem $placement, ProximityResult $result): array
    {
        $systems = $this->resolveSystems([$alert->target_solarsystem_id, $placement->solarsystem_id, ...$result->route]);

        $target = $systems[$alert->target_solarsystem_id] ?? ['name' => (string) $alert->target_solarsystem_id, 'security' => 0.0];
        $originName = $systems[$placement->solarsystem_id]['name'] ?? (string) $placement->solarsystem_id;
        $routeNames = array_map(fn (int $id): string => $systems[$id]['name'] ?? (string) $id, $result->route);

        return [
            'title' => sprintf('%s is now within %d %s', $target['name'], $result->jumps, $result->jumps === 1 ? 'jump' : 'jumps'),
            'description' => sprintf('**%s** was added to **%s**.', $originName, $alert->map->name),
            'color' => $this->colorForSecurity($target['security']),
            'fields' => [
                ['name' => 'Target', 'value' => $target['name'], 'inline' => true],
                ['name' => 'Map', 'value' => $alert->map->name, 'inline' => true],
                ['name' => 'Gate jumps', 'value' => (string) $result->jumps, 'inline' => true],
                ['name' => 'Route', 'value' => implode(' → ', $routeNames)],
            ],
        ];
    }
}
