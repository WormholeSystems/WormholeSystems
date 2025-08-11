<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard;

use App\Http\Integrations\zKillboard\DTO\RedisQKillmail;
use App\Http\Integrations\zKillboard\Requests\GetKill;
use App\Http\Integrations\zKillboard\Requests\GetSolarsystemKills;
use App\Http\Integrations\zKillboard\Requests\ListenForKill;
use Exception;

final class zKillboard
{
    /**
     * @throws Exception
     */
    public function getSolarsystemKills(int $solarsystem_id): array
    {
        $request = new GetSolarsystemKills($solarsystem_id);

        return new zKillboardConnector()->handle($request);
    }

    /**
     * @throws Exception
     */
    public function getKill(int $killmail_id): array
    {
        $request = new GetKill($killmail_id);

        return new zKillboardConnector()->handle($request);
    }

    /**
     * @throws Exception
     */
    public function listenForKill(string $identifier): ?RedisQKillmail
    {
        $request = new ListenForKill($identifier);

        return new zKillboardConnector()->handle($request);
    }
}
