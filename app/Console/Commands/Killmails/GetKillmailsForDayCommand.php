<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Console\Commands\AppCommand;
use App\Models\Alliance;
use App\Models\Corporation;
use App\Models\Killmail;
use Carbon\CarbonImmutable;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Storage;
use PharData;
use RecursiveIteratorIterator;

use function Laravel\Prompts\progress;

final class GetKillmailsForDayCommand extends AppCommand
{
    private const int CHUNK_SIZE = 500;

    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:get-killmails-for-day {date}';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Download and process killmails for a specific day';

    /**
     * Execute the console command.
     *
     * @throws ConnectionException
     */
    public function handle(): int
    {
        $date = CarbonImmutable::parse($this->argument('date'));

        $remote_file = $this->getRemoteFileName($date);
        $local_file = sprintf('killmails/%s.tar.bz2', $date->format('Y-m-d'));

        $this->info(sprintf('Downloading %s to %s', $remote_file, $local_file));

        Storage::makeDirectory('killmails');

        $response = Http::retry(5, throw: false)->sink(Storage::path($local_file))->get($remote_file);

        if ($response->failed()) {
            Storage::delete($local_file);

            if ($response->notFound()) {
                $this->info(sprintf('No killmails found for %s', $date->toDateString()));

                return self::SUCCESS;
            }
            $this->error(sprintf('Failed to download file from %s', $remote_file));

            return self::FAILURE;
        }

        $this->info(sprintf('Successfully downloaded %s', $local_file));

        $this->processKillmails(Storage::path($local_file));

        Storage::delete($local_file);

        return self::SUCCESS;
    }

    private function getRemoteFileName(CarbonImmutable $date): string
    {
        return sprintf(
            'https://data.everef.net/killmails/%s/killmails-%s-%s-%s.tar.bz2',
            $date->year,
            $date->year,
            $date->format('m'),
            $date->format('d')
        );
    }

    /** The upsert leaves `zkb` untouched so re-imports keep the live listener's data. */
    private function processKillmails(string $archive_path): void
    {
        $archive = new PharData($archive_path);
        $entries = array_keys(iterator_to_array(new RecursiveIteratorIterator($archive)));

        if ($entries === []) {
            $this->info('No killmail files to process.');

            return;
        }

        $corporation_ids = [];
        $alliance_ids = [];

        progress(
            label: 'Processing killmails...',
            steps: collect($entries)->chunk(self::CHUNK_SIZE),
            callback: function (Collection $chunk) use (&$corporation_ids, &$alliance_ids): void {
                $now = now();
                $rows = [];

                foreach ($chunk as $entry) {
                    $content = json_decode((string) file_get_contents($entry), true);
                    if (! $content) {
                        continue;
                    }

                    $rows[] = [
                        'id' => $content['killmail_id'],
                        'hash' => $content['killmail_hash'],
                        'time' => CarbonImmutable::parse($content['killmail_time'])->toDateTimeString(),
                        'solarsystem_id' => $content['solar_system_id'],
                        'data' => json_encode($content),
                        'zkb' => json_encode([]),
                        'created_at' => $now,
                        'updated_at' => $now,
                    ];

                    if ($corporation_id = $content['victim']['corporation_id'] ?? null) {
                        $corporation_ids[$corporation_id] = true;
                    }

                    if ($alliance_id = $content['victim']['alliance_id'] ?? null) {
                        $alliance_ids[$alliance_id] = true;
                    }
                }

                Killmail::query()->upsert($rows, ['id'], ['hash', 'time', 'solarsystem_id', 'data', 'updated_at']);
            },
        );

        $this->storeVictimOrganisations(array_keys($corporation_ids), array_keys($alliance_ids));
    }

    /**
     * Bare id rows only — the hourly sweep resolves the names, keeping ESI off the ingest path.
     *
     * @param  int[]  $corporation_ids
     * @param  int[]  $alliance_ids
     */
    private function storeVictimOrganisations(array $corporation_ids, array $alliance_ids): void
    {
        $now = now();
        $toRow = fn (int $id): array => ['id' => $id, 'created_at' => $now, 'updated_at' => $now];

        collect($corporation_ids)->chunk(self::CHUNK_SIZE)->each(
            fn (Collection $chunk) => Corporation::query()->insertOrIgnore($chunk->map($toRow)->all()),
        );

        collect($alliance_ids)->chunk(self::CHUNK_SIZE)->each(
            fn (Collection $chunk) => Alliance::query()->insertOrIgnore($chunk->map($toRow)->all()),
        );
    }
}
