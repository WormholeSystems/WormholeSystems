<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Enums\MassStatus;
use App\Models\Solarsystem;
use App\Services\RouteOptions;
use App\Services\RouteService;
use App\Utilities\CCPRounding;
use Closure;
use Illuminate\Console\Command;

use function Laravel\Prompts\search;
use function Laravel\Prompts\select;
use function Laravel\Prompts\table;
use function Laravel\Prompts\text;
use function Laravel\Prompts\warning;

final class FindClosestSystemsCommand extends Command
{
    protected $signature = 'app:closest-systems';

    protected $description = 'Test the findClosestSystems method with different conditions';

    public function handle(RouteService $routeService): int
    {
        $solarsystem_id = search('Enter the name of the starting solar system:', fn (string $query) => Solarsystem::query()
            ->whereLike('name', '%'.$query.'%')
            ->limit(10)
            ->pluck('name', 'id')
            ->all()
        );

        $fromSystem = Solarsystem::query()->find($solarsystem_id);

        $condition = select('Select the condition to filter systems:', [
            'observatories' => 'Systems with Jove Observatories',
            'highsec' => 'High-Security Systems (>= 0.5)',
            'lowsec' => 'Low-Security Systems (0.1 to 0.4)',
            'nullsec' => 'Null-Security Systems (<= 0.0)',
        ]);

        $limit = (int) text('Enter the maximum number of systems to find:', validate: fn (string $value): ?string => match (true) {
            ! is_numeric($value) || (int) $value <= 0 => 'Please enter a positive integer.',
            default => null,
        });

        $options = new RouteOptions(
            allowEol: true,
            massStatus: MassStatus::Fresh,
            allowEveScout: false
        );

        $conditionClosure = $this->getConditionClosure($condition);

        $results = $routeService->findClosestSystems($fromSystem->id, $options, $conditionClosure, $limit);
        if ($results === []) {
            warning('No systems found matching the criteria.');

            return self::SUCCESS;
        }

        table(
            headers: ['System Name', 'Region', 'Security', 'Distance (jumps)'],
            rows: array_map(fn (array $result): array => [
                $result['solarsystem']->name,
                $result['solarsystem']->region->name,
                number_format(CCPRounding::roundSecurity($result['solarsystem']->security), 1),
                $result['distance'],
            ], $results)
        );

        return self::SUCCESS;
    }

    private function getConditionClosure(string $condition): ?Closure
    {
        return match ($condition) {
            'observatories' => fn ($query) => $query->where('has_jove_observatory', true),
            'highsec' => fn ($query) => $query->where('security', '>=', 0.5),
            'lowsec' => fn ($query) => $query->whereBetween('security', [0.1, 0.4]),
            'nullsec' => fn ($query) => $query->where('security', '<=', 0.0),
            default => null,
        };
    }
}
