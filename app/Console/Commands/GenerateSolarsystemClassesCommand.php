<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Enums\SolarsystemClass;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\File;

final class GenerateSolarsystemClassesCommand extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'generate:solarsystem-classes';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Generate JSON file with solarsystem class metadata (labels, colours, groupings)';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $this->info('Generating solarsystem classes JSON file...');

        $classes = collect(SolarsystemClass::cases())
            ->map(fn (SolarsystemClass $class): array => [
                'value' => $class->value,
                'label' => $class->label(),
                'short_label' => $class->shortLabel(),
                'color_token' => $class->colorToken(),
                'sort_weight' => $class->sortWeight(),
                'is_standard' => $class->isStandard(),
                'is_special' => $class->isSpecial(),
                'is_drifter' => $class->isDrifter(),
                'is_known_space' => $class->isKnownSpace(),
                'is_wormhole_space' => $class->isWormholeSpace(),
            ])
            ->values();

        $data = [
            'generated_at' => now()->toIso8601String(),
            'classes' => $classes,
        ];

        $outputPath = resource_path('js/data/solarsystem_classes.json');
        File::put($outputPath, json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES));

        $this->info("✓ Generated solarsystem classes file at: {$outputPath}");
        $this->info("Total classes: {$classes->count()}");

        return Command::SUCCESS;
    }
}
