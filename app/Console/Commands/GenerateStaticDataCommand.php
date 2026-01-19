<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Models\Constellation;
use App\Models\Region;
use App\Models\Solarsystem;
use App\Models\Sovereignty;
use App\Models\WormholeStatic;
use App\Utilities\CCPRounding;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\File;
use NicolasKion\SDE\Models\SolarsystemConnection;
use RuntimeException;

use function collect;

final class GenerateStaticDataCommand extends Command
{
    protected $signature = 'generate:static-data';

    protected $description = 'Generate static data files for client-side routing';

    public function handle(): int
    {
        $this->info('Generating static data files...');

        $resourcesPath = resource_path('static');
        File::ensureDirectoryExists($resourcesPath);

        $storagePath = storage_path('app/static');
        File::ensureDirectoryExists($storagePath);

        $regions = Region::query()
            ->orderBy('id')
            ->get()
            ->map(fn (Region $region): array => [
                'id' => $region->id,
                'name' => $region->name,
                'type' => $region->type,
            ])
            ->values()
            ->all();

        $constellations = Constellation::query()
            ->with('region')
            ->orderBy('id')
            ->get()
            ->map(fn (Constellation $constellation): array => [
                'id' => $constellation->id,
                'name' => $constellation->name,
                'type' => $constellation->type,
                'region_id' => $constellation->region_id,
                'region' => [
                    'id' => $constellation->region->id,
                    'name' => $constellation->region->name,
                ],
            ])
            ->values()
            ->all();

        $solarsystems = Solarsystem::query()
            ->withCount('stations')
            ->with([
                'constellation',
                'region',
                'wormholeSystem.effect',
                'wormholeSystem.wormholeStatics.wormhole',
            ])
            ->orderBy('id')
            ->get()
            ->map(fn (Solarsystem $solarsystem): array => $this->formatSolarsystem($solarsystem))
            ->values()
            ->all();

        $connections = SolarsystemConnection::query()
            ->whereDoesntHaveRelation('fromSolarsystem', 'name', '=', 'Zarzakh')
            ->select('from_solarsystem_id', 'to_solarsystem_id')
            ->get()
            ->groupBy('from_solarsystem_id')
            ->map(static fn ($group): array => $group->pluck('to_solarsystem_id')->toArray())
            ->all();

        $sovereignties = Sovereignty::query()
            ->with(['alliance', 'corporation', 'faction'])
            ->orderBy('solarsystem_id')
            ->get()
            ->mapWithKeys(fn (Sovereignty $sovereignty): array => [
                $sovereignty->solarsystem_id => $this->formatSovereignty($sovereignty),
            ])
            ->all();

        $this->writePlainJson("{$resourcesPath}/regions.json", $regions);
        $this->writePlainJson("{$resourcesPath}/constellations.json", $constellations);
        $this->writePlainJson("{$resourcesPath}/solarsystems.json", $solarsystems);
        $this->writePlainJson("{$resourcesPath}/connections.json", $connections);
        $this->writeCompressedJson("{$storagePath}/sovereignty.json.gz", $sovereignties);

        $this->info("✓ Static data written to {$resourcesPath}");
        $this->info("✓ Sovereignty data written to {$storagePath}");

        return Command::SUCCESS;
    }

    private function formatSolarsystem(Solarsystem $solarsystem): array
    {
        return [
            'id' => $solarsystem->id,
            'name' => $solarsystem->name,
            'region_id' => $solarsystem->region_id,
            'constellation_id' => $solarsystem->constellation_id,
            'class' => $solarsystem->wormholeSystem?->class,
            'security' => CCPRounding::roundSecurity($solarsystem->security),
            'type' => $solarsystem->type,
            'region' => [
                'id' => $solarsystem->region->id,
                'name' => $solarsystem->region->name,
            ],
            'constellation' => [
                'id' => $solarsystem->constellation->id,
                'name' => $solarsystem->constellation->name,
            ],
            'has_jove_observatory' => $solarsystem->has_jove_observatory,
            'has_stations' => $solarsystem->stations_count > 0,
            'statics' => $solarsystem->wormholeSystem?->wormholeStatics?->map(
                static fn (WormholeStatic $static): array => [
                    'id' => $static->wormhole->id,
                    'leads_to' => $static->wormhole->leads_to,
                    'name' => $static->wormhole->name,
                    'maximum_lifetime' => $static->wormhole->maximum_lifetime,
                    'maximum_jump_mass' => $static->wormhole->maximum_jump_mass,
                    'total_mass' => $static->wormhole->total_mass,
                ]
            )->values()->all(),
            'effect' => $this->getWormholeEffects($solarsystem),
            'position2D' => $this->getPosition2dArray($solarsystem),
        ];

    }

    private function formatSovereignty(Sovereignty $sovereignty): array
    {
        return [
            'id' => $sovereignty->id,
            'alliance' => $this->formatAlliance($sovereignty),
            'corporation' => $this->formatCorporation($sovereignty),
            'faction' => $this->formatFaction($sovereignty),
        ];
    }

    private function formatAlliance(Sovereignty $sovereignty): ?array
    {
        if ($sovereignty->alliance === null) {
            return null;
        }

        return [
            'id' => $sovereignty->alliance->id,
            'name' => $sovereignty->alliance->name,
            'ticker' => $sovereignty->alliance->ticker,
        ];
    }

    private function formatCorporation(Sovereignty $sovereignty): ?array
    {
        if ($sovereignty->corporation === null) {
            return null;
        }

        return [
            'id' => $sovereignty->corporation->id,
            'name' => $sovereignty->corporation->name,
            'ticker' => $sovereignty->corporation->ticker,
        ];
    }

    private function formatFaction(Sovereignty $sovereignty): ?array
    {
        if ($sovereignty->faction === null) {
            return null;
        }

        return [
            'id' => $sovereignty->faction->id,
            'name' => $sovereignty->faction->name,
        ];
    }

    private function getWormholeEffects(Solarsystem $solarsystem): ?array
    {
        $effects = $solarsystem->wormholeSystem?->effect?->effects;

        if (! $effects) {
            return null;
        }

        ['Buffs' => $buffs, 'Debuffs' => $debuffs] = $effects;

        $strength = $solarsystem->wormholeSystem->class - 1;

        $buffs = collect($buffs)
            ->map(static fn (array $strengths, string $name): array => [
                'name' => $name,
                'strength' => $strengths[$strength] ?? null,
                'type' => 'Buff',
            ])
            ->merge(
                collect($debuffs)
                    ->map(static fn (array $strengths, string $name): array => [
                        'name' => $name,
                        'strength' => $strengths[$strength] ?? null,
                        'type' => 'Debuff',
                    ])
            )
            ->values()
            ->all();

        return [
            'id' => $solarsystem->wormholeSystem->effect->id,
            'name' => $solarsystem->wormholeSystem->effect->name,
            'effects' => $buffs,
        ];
    }

    private function getPosition2dArray(Solarsystem $solarsystem): ?array
    {
        if ($solarsystem->pos_2d_x === null || $solarsystem->pos_2d_y === null) {
            return null;
        }

        return [
            'x' => $solarsystem->pos_2d_x,
            'y' => $solarsystem->pos_2d_y,
        ];
    }

    private function writeCompressedJson(string $path, array $data): void
    {
        $encoded = json_encode($data, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES);
        $compressed = gzencode($encoded, 9);

        if ($compressed === false) {
            throw new RuntimeException("Failed to compress JSON for {$path}.");
        }

        File::put($path, $compressed);
    }

    private function writePlainJson(string $path, array $data): void
    {
        $encoded = json_encode($data, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT);
        File::put($path, $encoded);
    }
}
