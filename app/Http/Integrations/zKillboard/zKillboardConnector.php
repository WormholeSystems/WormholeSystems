<?php

declare(strict_types=1);

namespace App\Http\Integrations\zKillboard;

use App\Enums\RequestMethod;
use Exception;
use Illuminate\Container\Attributes\Config;
use Illuminate\Support\Facades\Http;
use InvalidArgumentException;

final readonly class zKillboardConnector
{
    public function __construct(
        #[Config('services.zkillboard.retry_attempts')] private int $retry_attempts,
        #[Config('services.zkillboard.retry_delay_ms')] private int $retry_delay_ms,
    ) {}

    public function handle(zKillboardRequest $zKillboardRequest): mixed
    {
        $method = $zKillboardRequest->method;
        $endpoint = $zKillboardRequest->getEndpoint();
        $base_url = $zKillboardRequest->base_url ?? 'https://zkillboard.com/api';

        $request = Http::baseUrl($base_url)
            ->retry($this->retry_attempts, $this->retry_delay_ms)
            ->withHeaders([
                'Accept' => 'application/json',
                'User-Agent' => config()->string('esi.user_agent'),
            ]);

        $result = match ($method) {
            RequestMethod::GET => $request->get($endpoint, $zKillboardRequest->getQuery()),
            default => throw new InvalidArgumentException('Unsupported request method: '.$method->value),
        };

        if ($result->notFound()) {
            return $zKillboardRequest->toDto(null);
        }

        if ($result->failed()) {
            throw new Exception('Request failed with status code: '.$result->status());
        }

        $data = $result->json();

        return $zKillboardRequest->toDto($data);
    }
}
