<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard;

use App\Http\Integrations\zKillboard\DTO\R2Z2Killmail;
use App\Http\Integrations\zKillboard\Requests\GetKill;
use App\Http\Integrations\zKillboard\Requests\GetR2Z2Killmail;
use App\Http\Integrations\zKillboard\Requests\GetR2Z2Sequence;
use App\Http\Integrations\zKillboard\Requests\GetSolarsystemKills;
use Exception;

final readonly class zKillboard
{
    public function __construct(
        private zKillboardConnector $connector,
    ) {}

    /**
     * @throws Exception
     */
    public function getSolarsystemKills(int $solarsystem_id): array
    {
        $request = new GetSolarsystemKills($solarsystem_id);

        return $this->connector->handle($request);
    }

    /**
     * @throws Exception
     */
    public function getKill(int $killmail_id): array
    {
        $request = new GetKill($killmail_id);

        return $this->connector->handle($request);
    }

    /**
     * @throws Exception
     */
    public function getLatestSequence(): int
    {
        $request = new GetR2Z2Sequence;

        return $this->connector->handle($request);
    }

    /**
     * @throws Exception
     */
    public function getKillmailBySequence(int $sequence_id): ?R2Z2Killmail
    {
        $request = new GetR2Z2Killmail($sequence_id);

        return $this->connector->handle($request);
    }
}
