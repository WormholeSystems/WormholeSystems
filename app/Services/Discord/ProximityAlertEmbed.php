<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\MapAlert;
use App\Services\Routing\ProximityResult;

/**
 * Builds the Discord embed for a proximity alert: a target system has just come within
 * range of the map. Coloured by the target's security status.
 */
final readonly class ProximityAlertEmbed
{
    use ColorsBySecurity;
    use ResolvesSolarsystems;

    /**
     * @return array<string, mixed>
     */
    public function build(MapAlert $alert, ProximityResult $result): array
    {
        $systems = $this->resolveSystems([
            $alert->target_solarsystem_id,
            $result->matchedOriginSolarsystemId,
            ...$result->route,
        ]);

        $target = $systems[$alert->target_solarsystem_id] ?? ['name' => (string) $alert->target_solarsystem_id, 'security' => 0.0];
        $originName = $systems[$result->matchedOriginSolarsystemId]['name'] ?? (string) $result->matchedOriginSolarsystemId;
        $routeNames = array_map(fn (int $id): string => $systems[$id]['name'] ?? (string) $id, $result->route);

        return [
            'title' => sprintf('Found new %d %s %s connection', $result->jumps, $result->jumps === 1 ? 'jump' : 'jumps', $target['name']),
            'url' => route('maps.show', $alert->map),
            'description' => sprintf('**%s** was just added to your map, putting **%s** within range.', $originName, $target['name']),
            'color' => $this->colorForSecurity((float) $target['security']),
            'fields' => [
                ['name' => 'Target', 'value' => sprintf('%s (%.1f)', $target['name'], $target['security']), 'inline' => true],
                ['name' => 'Gate jumps', 'value' => (string) $result->jumps, 'inline' => true],
                ['name' => 'From system', 'value' => $originName, 'inline' => true],
                ['name' => 'Route', 'value' => implode(' → ', $routeNames)],
            ],
        ];
    }
}
