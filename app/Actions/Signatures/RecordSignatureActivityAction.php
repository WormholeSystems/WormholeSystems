<?php

declare(strict_types=1);

namespace App\Actions\Signatures;

use App\Enums\SignatureActivityAction;
use App\Models\Character;
use App\Models\Signature;
use App\Models\SignatureActivity;
use Carbon\CarbonImmutable;

final readonly class RecordSignatureActivityAction
{
    private const int CHURN_WINDOW_SECONDS = 60;

    public function handle(Signature $signature, ?Character $actor, SignatureActivityAction $action): void
    {
        if ($actor === null) {
            return;
        }

        if ($signature->is_anomaly) {
            return;
        }

        $map_solarsystem = $signature->mapSolarsystem;

        // Take the clock exactly once: the churn window and the day bucket must never
        // straddle two clocks.
        $at = CarbonImmutable::now('UTC');

        if ($action === SignatureActivityAction::Deleted) {
            $matching_created = SignatureActivity::query()
                ->where('map_id', $map_solarsystem->map_id)
                ->where('character_id', $actor->id)
                // Keyed on the signature PK, not dedupe_key: signature_id (the in-game code)
                // is a nullable, editable string, so it can change mid-life and make the
                // dedupe key a false miss for this lookup.
                ->where('signature_id', $signature->id)
                ->where('action', SignatureActivityAction::Created->value)
                ->where('created_at', '>=', $at->subSeconds(self::CHURN_WINDOW_SECONDS))
                ->first();

            if ($matching_created !== null) {
                $matching_created->delete();

                return;
            }
        }

        $dedupe_key = mb_substr(
            $signature->signature_id !== null && $signature->signature_id !== '' ? $signature->signature_id : 'pk:'.$signature->id,
            0,
            64,
        );

        // insertOrIgnore compiles to MySQL INSERT IGNORE, which downgrades truncation and FK
        // violations to warnings and silently drops the row -- not just unique-key collisions.
        // Benign here since the payload is complete and dedupe_key is already truncated, but
        // worth knowing before a future silent drop becomes undiagnosable.
        SignatureActivity::query()->insertOrIgnore([
            'map_id' => $map_solarsystem->map_id,
            'character_id' => $actor->id,
            'signature_id' => $signature->id,
            'solarsystem_id' => $map_solarsystem->solarsystem_id,
            'dedupe_key' => $dedupe_key,
            'action' => $action->value,
            'activity_date' => $at->toDateString(),
            'created_at' => $at->format('Y-m-d H:i:s'),
        ]);
    }
}
