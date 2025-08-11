<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard;

use App\Enums\RequestMethod;

abstract class zKillboardRequest
{
    public RequestMethod $method;

    public ?string $base_url = null;

    abstract public function getEndpoint(): string;

    public function getQuery(): array
    {
        return [];
    }

    public function toDto(?array $data): mixed
    {
        return $data;
    }
}
