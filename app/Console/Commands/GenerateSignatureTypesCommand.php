<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Models\SignatureCategory;
use App\Models\SignatureType;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\File;

final class GenerateSignatureTypesCommand extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'generate:signature-types';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Generate JSON file with all signature categories and types';

    /**
     * Execute the console command.
     */
    public function handle(): int
    {
        $this->info('Generating signature types JSON file...');

        $categories = SignatureCategory::query()
            ->orderBy('id')
            ->get()
            ->map(fn ($cat): array => [
                'id' => $cat->id,
                'name' => $cat->name,
                'code' => $cat->code,
            ]);

        $types = SignatureType::query()
            ->with('category')
            ->orderBy('signature_category_id')
            ->orderBy('name')
            ->get()
            ->map(fn ($type): array => [
                'id' => $type->id,
                'name' => $type->name,
                'signature' => $type->signature,
                'signature_category_id' => $type->signature_category_id,
                'category_name' => $type->category->name,
                'target_class' => $type->target_class,
                'extra' => $type->extra,
                'spawn_areas' => $type->spawn_areas,
            ]);

        $data = [
            'generated_at' => now()->toIso8601String(),
            'categories' => $categories,
            'types' => $types,
        ];

        $outputPath = resource_path('js/data/signatures.json');
        File::put($outputPath, json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES));

        $this->info("✓ Generated signature types file at: {$outputPath}");
        $this->info("Total categories: {$categories->count()}");
        $this->info("Total types: {$types->count()}");

        return Command::SUCCESS;
    }
}
