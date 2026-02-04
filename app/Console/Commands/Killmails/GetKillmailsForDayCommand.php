<?php

declare(strict_types=1);

namespace App\Console\Commands\Killmails;

use App\Actions\EnsureOrganisationExistsAction;
use App\Console\Commands\AppCommand;
use App\Models\Killmail;
use Carbon\CarbonImmutable;
use Illuminate\Http\Client\ConnectionException;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Process;
use Illuminate\Support\Facades\Storage;

use function Laravel\Prompts\progress;

final class GetKillmailsForDayCommand extends AppCommand
{
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
    public function handle(EnsureOrganisationExistsAction $ensureOrganisationExistsAction): int
    {
        $date = CarbonImmutable::parse($this->argument('date'));

        $remote_file = $this->getRemoteFileName($date);
        $local_file = $this->getLocalFileName($date);

        $this->info(sprintf('Downloading %s to %s', $remote_file, $local_file));

        $response = Http::retry(5, throw: false)->get($remote_file);

        if ($response->failed()) {
            if ($response->notFound()) {
                $this->info(sprintf('No killmails found for %s', $date->toDateString()));

                return self::SUCCESS;
            }
            $this->error(sprintf('Failed to download file from %s', $remote_file));

            return self::FAILURE;
        }

        Storage::put($local_file, $response->body());

        $this->info(sprintf('Successfully downloaded %s', $local_file));

        $this->extractKillmails($local_file);

        $this->processKillmails($ensureOrganisationExistsAction);

        Storage::delete($local_file);
        Storage::deleteDirectory('killmails/killmails');

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

    private function getLocalFileName(CarbonImmutable $date): string
    {
        return sprintf(
            '%s/%s.tar.bz2',
            Storage::path('killmails'),
            $date->format('Y-m-d')
        );
    }

    private function extractKillmails(string $local_file): void
    {

        $this->info(sprintf('Extracting %s', $local_file));

        $extracted_path = Storage::path('killmails/');
        if (! Storage::exists($extracted_path)) {
            Storage::makeDirectory('killmails/');
        }

        $command = sprintf('tar -xjf %s -C %s', Storage::path($local_file), $extracted_path);

        $res = Process::run($command);

        if ($res->successful()) {
            $this->info(sprintf('Successfully extracted %s', $local_file));
        } else {
            $this->error(sprintf('Failed to extract %s: %s', $local_file, $res->errorOutput()));
        }

    }

    private function processKillmails(EnsureOrganisationExistsAction $ensureOrganisationExistsAction): void
    {
        $files = Storage::files('killmails/killmails');

        if (empty($files)) {
            $this->info('No killmail files to process.');

            return;
        }

        progress(
            label: 'Processing killmails...',
            steps: $files,
            callback: function (string $file) use ($ensureOrganisationExistsAction): void {
                $content = Storage::json($file);
                if (! $content) {
                    return;
                }

                Killmail::query()->firstOrCreate(
                    [
                        'id' => $content['killmail_id'],
                        'hash' => $content['killmail_hash'],
                    ],
                    [
                        'time' => $content['killmail_time'],
                        'solarsystem_id' => $content['solar_system_id'],
                        'data' => $content,
                        'zkb' => [],
                    ]
                );

                $ensureOrganisationExistsAction->ensureCorporationExists($content['victim']['corporation_id'] ?? null);
                $ensureOrganisationExistsAction->ensureAllianceExists($content['victim']['alliance_id'] ?? null);
            },
        );
    }
}
