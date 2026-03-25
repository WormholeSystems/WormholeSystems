<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Enums\MapSolarsystemStatus;
use App\Models\MapSolarsystem;

use function Laravel\Prompts\info;
use function Laravel\Prompts\progress;

final class PurgeKillmailNotes extends AppCommand
{
    protected $signature = 'app:purge-killmail-notes';

    protected $description = 'Remove killmail analysis sections from map solarsystem notes while preserving user-written content';

    public function handle(): int
    {
        $query = MapSolarsystem::query()
            ->whereNotNull('notes')
            ->where('notes', 'LIKE', '%<!-- killmails:start -->%');

        $count = $query->count();

        if ($count === 0) {
            info('No solarsystems with killmail notes found.');

            return self::SUCCESS;
        }

        progress('Purging killmail notes', $query->lazyById(), function (MapSolarsystem $solarsystem): void {
            $cleaned = preg_replace(
                '/<!-- killmails:start -->.*?<!-- killmails:end -->/s',
                '',
                $solarsystem->notes
            );

            $cleaned = mb_trim($cleaned);

            $solarsystem->update([
                'notes' => $cleaned !== '' ? $cleaned : null,
                'status' => MapSolarsystemStatus::Unknown,
            ]);
        });

        info(sprintf('Purged killmail notes from %d solarsystem(s).', $count));

        return self::SUCCESS;
    }
}
