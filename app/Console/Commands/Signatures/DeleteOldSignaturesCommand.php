<?php

declare(strict_types=1);

namespace App\Console\Commands\Signatures;

use App\Actions\Signatures\DeleteSignatureAction;
use App\Console\Commands\AppCommand;
use App\Enums\SignatureCategory;
use App\Models\Signature;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\DB;
use Throwable;

final class DeleteOldSignaturesCommand extends AppCommand
{
    public const int SIGNATURE_LIFETIME_DAYS = 7;

    public const int WORMHOLE_SIGNATURE_LIFETIME_DAYS = 3;

    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:delete-old-signatures';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Deletes old signatures from the database';

    public function __construct(
        private readonly DeleteSignatureAction $deleteSignatureAction
    ) {
        parent::__construct();
    }

    /**
     * Execute the console command.
     *
     * @throws Throwable
     */
    public function handle(): int
    {
        $this->deleteWormholeSignatures();
        $this->deleteUncategorizedSignatures();

        return self::SUCCESS;
    }

    /**
     * @throws Throwable
     */
    private function deleteUncategorizedSignatures(): void
    {
        $old_signatures = Signature::query()
            ->where('created_at', '<', now()->subDays(self::SIGNATURE_LIFETIME_DAYS))
            ->get();

        if ($old_signatures->isEmpty()) {
            $this->info('No old signatures found to delete.');

            return;
        }

        DB::transaction(function () use ($old_signatures): void {
            foreach ($old_signatures as $signature) {
                $this->deleteSignatureAction->handle($signature, without_events: true);
            }
        });

        $this->info(sprintf(
            'Deleted %d old signatures successfully.',
            $old_signatures->count()
        ));
    }

    /**
     * @throws Throwable
     */
    private function deleteWormholeSignatures(): void
    {
        $old_signatures = Signature::query()
            ->whereRelation('signatureCategory', 'code', SignatureCategory::Wormhole)
            ->where('created_at', '<', now()->subDays(self::WORMHOLE_SIGNATURE_LIFETIME_DAYS))
            ->get();

        if ($old_signatures->isEmpty()) {
            $this->info('No old wormhole signatures found to delete.');

            return;
        }

        DB::transaction(function () use ($old_signatures): void {
            foreach ($old_signatures as $signature) {
                $this->deleteSignatureAction->handle($signature, without_events: true);
            }
        });

        $this->info(sprintf(
            'Deleted %d old wormhole signatures successfully.',
            $old_signatures->count()
        ));
    }
}
