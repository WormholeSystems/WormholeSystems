<?php

namespace App\Http\Integrations\zKillboard\Requests;

use App\Enums\RequestMethod;
use App\Http\Integrations\zKillboard\DTO\RedisQKillmail;
use App\Http\Integrations\zKillboard\zKillboardRequest;

class ListenForKill extends zKillboardRequest
{
    public RequestMethod $method = RequestMethod::GET;

    public ?string $base_url = 'https://zkillredisq.stream/';

    public function __construct(
        public string $identifier,
    ) {}

    public function getEndpoint(): string
    {
        return 'listen.php';
    }

    public function getQuery(): array
    {
        return [
            'queueID' => $this->identifier,
        ];
    }

    public function toDto(?array $data): ?RedisQKillmail
    {
        $package = $data['package'] ?? null;

        if (is_null($package)) {
            return null;
        }

        return RedisQKillmail::fromArray($package);
    }
}
