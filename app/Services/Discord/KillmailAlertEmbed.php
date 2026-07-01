<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\Killmail;
use App\Models\MapAlert;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Models\Type;
use App\Services\Killmails\KillmailFilterDescriber;
use App\Services\Killmails\KillmailFilterRule;
use App\Services\NameResolver;
use App\Services\Routing\ProximityResult;
use Illuminate\Support\Collection;

/**
 * Builds the Discord embed for a killmail alert. Resolves the victim and the final-blow
 * attacker to real names so the alert reads like a sentence, e.g. "Foo (Corp · Alliance)
 * lost a Raven in Jita to a Raichu flown by Bar".
 */
final readonly class KillmailAlertEmbed
{
    use ColorsBySecurity;

    public function __construct(
        private KillmailFilterDescriber $filterDescriber,
        private NameResolver $nameResolver,
    ) {}

    /**
     * @param  Collection<int, KillmailFilterRule>  $matchedFilters
     * @return array<string, mixed>
     */
    public function build(MapAlert $alert, Killmail $killmail, ProximityResult $result, Collection $matchedFilters): array
    {
        $data = json_decode(json_encode($killmail->data), true) ?: [];
        $victim = is_array($data['victim'] ?? null) ? $data['victim'] : [];
        $attackers = is_array($data['attackers'] ?? null) ? $data['attackers'] : [];
        $finalBlow = $this->finalBlow($attackers);

        $shipTypeId = (int) ($victim['ship_type_id'] ?? 0);
        $names = $this->nameResolver->resolve([
            (int) ($victim['character_id'] ?? 0),
            (int) ($victim['corporation_id'] ?? 0),
            (int) ($victim['alliance_id'] ?? 0),
            (int) ($finalBlow['character_id'] ?? 0),
        ]);

        $systems = $this->routeSystems($result);
        $systemName = $systems[$killmail->solarsystem_id]->name ?? (string) $killmail->solarsystem_id;
        $security = (float) ($systems[$killmail->solarsystem_id]->security ?? 0.0);
        $shipName = $this->typeName($shipTypeId);

        $pilot = $names[(int) ($victim['character_id'] ?? 0)] ?? null;
        $affiliation = $this->affiliation($names, $victim);
        $attackerName = $names[(int) ($finalBlow['character_id'] ?? 0)] ?? null;
        $attackerShip = ($id = (int) ($finalBlow['ship_type_id'] ?? 0)) > 0 ? $this->typeName($id) : null;

        $embed = [
            'title' => sprintf('%s killed in %s', $shipName, $systemName),
            'url' => sprintf('https://zkillboard.com/kill/%d/', $killmail->id),
            'description' => $this->description($result, $systemName, $shipName, $pilot, $affiliation, $attackerName, $attackerShip),
            'color' => $this->colorForSecurity($security),
            'fields' => $this->fields($alert, $killmail, $result, $systems, $systemName, $security, $shipName, $pilot, $affiliation, $attackerName, $attackerShip, $matchedFilters),
        ];

        if ($killmail->time !== null) {
            $embed['timestamp'] = $killmail->time->toIso8601String();
        }

        if ($shipTypeId > 0) {
            $embed['thumbnail'] = ['url' => sprintf('https://images.evetech.net/types/%d/render?size=128', $shipTypeId)];
        }

        return $embed;
    }

    /**
     * @param  array<int, mixed>  $attackers
     * @return array<string, mixed>
     */
    private function finalBlow(array $attackers): array
    {
        $finalBlow = collect($attackers)->first(fn ($attacker): bool => is_array($attacker) && ($attacker['final_blow'] ?? false) === true);

        return is_array($finalBlow) ? $finalBlow : [];
    }

    /**
     * @param  array<int, string>  $names
     * @param  array<string, mixed>  $victim
     */
    private function affiliation(array $names, array $victim): string
    {
        return collect([
            $names[(int) ($victim['corporation_id'] ?? 0)] ?? null,
            $names[(int) ($victim['alliance_id'] ?? 0)] ?? null,
        ])->filter()->implode(' · ');
    }

    /**
     * The human-readable lead line plus the range sentence.
     */
    private function description(ProximityResult $result, string $systemName, string $shipName, ?string $pilot, string $affiliation, ?string $attackerName, ?string $attackerShip): string
    {
        $subject = $pilot !== null
            ? sprintf('**%s**%s', $pilot, $affiliation !== '' ? sprintf(' (%s)', $affiliation) : '')
            : sprintf('A **%s**', $shipName);

        $loss = $pilot !== null
            ? sprintf('%s lost a **%s** in **%s**%s.', $subject, $shipName, $systemName, $this->killedBy($attackerName, $attackerShip))
            : sprintf('%s was destroyed in **%s**%s.', $subject, $systemName, $this->killedBy($attackerName, $attackerShip));

        $range = $result->jumps === 0
            ? 'A matching killmail occurred **inside your chain**.'
            : sprintf('A matching killmail occurred **%d %s** from your chain.', $result->jumps, $result->jumps === 1 ? 'jump' : 'jumps');

        return $loss.' '.$range;
    }

    private function killedBy(?string $attackerName, ?string $attackerShip): string
    {
        return match (true) {
            $attackerName !== null && $attackerShip !== null => sprintf(' to a **%s** flown by **%s**', $attackerShip, $attackerName),
            $attackerName !== null => sprintf(' to **%s**', $attackerName),
            $attackerShip !== null => sprintf(' to a **%s**', $attackerShip),
            default => '',
        };
    }

    /**
     * @param  Collection<int, Solarsystem>  $systems
     * @param  Collection<int, KillmailFilterRule>  $matchedFilters
     * @return array<int, array<string, mixed>>
     */
    private function fields(MapAlert $alert, Killmail $killmail, ProximityResult $result, Collection $systems, string $systemName, float $security, string $shipName, ?string $pilot, string $affiliation, ?string $attackerName, ?string $attackerShip, Collection $matchedFilters): array
    {
        $totalValue = (float) (data_get($killmail->zkb, 'totalValue') ?? 0.0);

        $fields = [
            ['name' => 'System', 'value' => sprintf('%s (%.1f)', $systemName, $security), 'inline' => true],
            ['name' => 'Ship', 'value' => $shipName, 'inline' => true],
            ['name' => 'Value', 'value' => sprintf('%s ISK', number_format($totalValue)), 'inline' => true],
        ];

        if ($pilot !== null) {
            $fields[] = ['name' => 'Pilot', 'value' => $pilot, 'inline' => true];
        }

        if ($affiliation !== '') {
            $fields[] = ['name' => 'Corp / Alliance', 'value' => $affiliation, 'inline' => true];
        }

        $finalBlowLabel = collect([$attackerName, $attackerShip])->filter()->implode(' — ');
        if ($finalBlowLabel !== '') {
            $fields[] = ['name' => 'Final blow', 'value' => $finalBlowLabel, 'inline' => true];
        }

        if ($killmail->time !== null) {
            $fields[] = ['name' => 'When', 'value' => sprintf('<t:%d:R>', $killmail->time->timestamp), 'inline' => true];
        }

        if ($result->jumps > 0) {
            $fields[] = ['name' => 'Exit from', 'value' => $this->resolveExit($alert, $result, $systems), 'inline' => true];
            $fields[] = ['name' => 'Jumps from chain', 'value' => (string) $result->jumps, 'inline' => true];
            $fields[] = ['name' => 'Route', 'value' => implode(' → ', array_map(fn (int $id): string => $systems[$id]->name ?? (string) $id, $result->route))];
        }

        $filterLines = $this->filterDescriber->describe($matchedFilters);
        if ($filterLines !== []) {
            $fields[] = ['name' => 'Matched filters', 'value' => implode("\n", $filterLines)];
        }

        return $fields;
    }

    /**
     * The chain system nearest the kill — where a pilot would exit the map — shown with
     * its map alias when one is set.
     *
     * @param  Collection<int, Solarsystem>  $systems
     */
    private function resolveExit(MapAlert $alert, ProximityResult $result, Collection $systems): string
    {
        $originId = $result->matchedOriginSolarsystemId;
        $name = $systems[$originId]->name ?? (string) $originId;

        $alias = MapSolarsystem::query()
            ->where('map_id', $alert->map_id)
            ->isSolarsystem($originId)
            ->value('alias');

        return $alias ? sprintf('%s (%s)', $alias, $name) : $name;
    }

    /**
     * @return Collection<int, Solarsystem>
     */
    private function routeSystems(ProximityResult $result): Collection
    {
        return Solarsystem::query()
            ->whereIn('id', $result->route)
            ->get(['id', 'name', 'security'])
            ->keyBy('id');
    }

    private function typeName(int $typeId): string
    {
        if ($typeId === 0) {
            return 'Unknown ship';
        }

        return Type::query()->find($typeId)?->name ?? (string) $typeId;
    }
}
