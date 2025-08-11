<?php

declare(strict_types=1);

namespace App\Exception;

use App\Http\Middleware\HandleInertiaRequests;
use Exception;
use Illuminate\Foundation\Configuration\Exceptions;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Log;
use Inertia\Inertia;
use Symfony\Component\HttpFoundation\Response;
use Throwable;

final class ExceptionHandler
{
    public function __invoke(Exceptions $exceptions): void
    {
        $exceptions->respond(function (Response $response, Throwable $exception, Request $request): Response {
            $status_code = $response->getStatusCode();

            return match (true) {
                $request->is('api/*') => $this->handleApiException($response),
                $this->shouldDisplayDebugInfo($status_code) => $response,
                $this->isSessionExpired($status_code) => $this->renderSessionError(),
                $this->isInMaintenanceMode() => $this->renderMaintenanceView($request, $status_code),
                $this->isExpectedError($status_code) => $this->renderErrorView($request, $response, $exception, $status_code),
                default => $response,
            };
        });
    }

    private function handleApiException(Response $response): Response
    {
        return response()->json([
            'message' => match ($response->getStatusCode()) {
                403 => 'Unauthorized',
                404 => 'Not Found',
                500 => 'Internal Server Error',
                default => 'An error occurred',
            },
            'status' => $response->getStatusCode(),
        ], $response->getStatusCode());
    }

    private function shouldDisplayDebugInfo(int $status_code): bool
    {
        return $this->isLocalEnvironment() && in_array($status_code, [500, 503]);
    }

    private function isLocalEnvironment(): bool
    {
        return app()->environment('local');
    }

    private function isSessionExpired(int $status_code): bool
    {
        return $status_code === 419;
    }

    private function renderSessionError(): Response
    {
        return back()->notify(
            'Your session has expired. Please refresh the page and try again.',
            'error'
        );
    }

    private function isInMaintenanceMode(): bool
    {
        return app()->isDownForMaintenance();
    }

    private function renderMaintenanceView(Request $request, int $status_code): Response
    {
        return Inertia::render('errors/ServiceUnavailable', [
            'discord_invite' => config('services.discord.invite'),
        ])
            ->toResponse($request)
            ->setStatusCode($status_code);
    }

    private function isExpectedError(int $status_code): bool
    {
        return in_array($status_code, [400, 401, 403, 404, 500]);
    }

    private function renderErrorView(Request $request, Response $response, Throwable $exception, int $status_code): Response
    {
        try {
            $component = match ($status_code) {
                400 => 'errors/BadRequest',
                401, 403 => 'errors/Unauthorized',
                404 => 'errors/NotFound',
                default => 'errors/InternalServerError',
            };

            Inertia::share((new HandleInertiaRequests)->share($request));

            return Inertia::render($component, [
                'status' => $status_code,
                'query' => $request->path(),
                'message' => match ($status_code) {
                    400 => $exception->getMessage(),
                    403 => 'Unauthorized',
                    404 => 'Not Found',
                    500 => 'Internal Server Error',
                    default => 'Unknown Error',
                },
                'discord_invite' => config('services.discord.invite'),
            ])
                ->toResponse($request)
                ->setStatusCode($status_code);
        } catch (Exception $e) {
            Log::info(sprintf('Failed to render error page: %s', $e->getMessage()));

            return $response;
        }
    }
}
