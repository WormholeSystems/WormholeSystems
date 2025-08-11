<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard\Requests;

use App\Enums\RequestMethod;
use App\Http\Integrations\zKillboard\zKillboardRequest;

final class GetKill extends zKillboardRequest
{
    public RequestMethod $method = RequestMethod::GET;

    public function __construct(
        public int $killmail_id,
    ) {}

    public function getEndpoint(): string
    {
        return sprintf('/killID/%d/', $this->killmail_id);
    }

    public function getQuery(): array
    {
        return [];
    }

    public function toDto(?array $data): ?array
    {
        return $data[0] ?? null;
    }
}
