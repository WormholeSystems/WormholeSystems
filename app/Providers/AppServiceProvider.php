<?php

declare(strict_types=1);

namespace App\Providers;

use App\DTO\CTA;
use App\Models\ServerStatus;
use App\Policies\PersonalAccessTokenPolicy;
use App\Services\RouteService;
use Carbon\CarbonImmutable;
use Illuminate\Console\Scheduling\Event as ScheduledEvent;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Date;
use Illuminate\Support\Facades\Event;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Facades\URL;
use Illuminate\Support\Facades\Vite;
use Illuminate\Support\ServiceProvider;
use Laravel\Sanctum\PersonalAccessToken;
use SocialiteProviders\Eveonline\Provider;
use SocialiteProviders\Manager\SocialiteWasCalled;

use function Laravel\Prompts\info;

final class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        $this->app->singleton(RouteService::class);
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        JsonResource::withoutWrapping();

        $this->registerNotificationMacro();

        Model::shouldBeStrict();
        Model::unguard();
        Model::automaticallyEagerLoadRelationships();

        Vite::useAggressivePrefetching();

        URL::forceHttps();

        Date::use(CarbonImmutable::class);

        Event::listen(function (SocialiteWasCalled $event): void {
            $event->extendSocialite('eveonline', Provider::class);
        });

        Gate::policy(PersonalAccessToken::class, PersonalAccessTokenPolicy::class);

        $this->registerScheduleMacros();
    }

    private function registerNotificationMacro(): void
    {
        RedirectResponse::macro('notify', function (string $title, string $message = '', string $type = 'success', ?CTA $action = null): RedirectResponse {
            if (request()->boolean('silent')) {
                return $this;
            }

            $notification = [
                'title' => $title,
                'message' => $message,
                'type' => $type,
            ];

            if ($action instanceof CTA) {
                $notification['action'] = $action->toArray();
            }

            return $this->with('notification', $notification);
        });
    }

    private function registerScheduleMacros(): void
    {
        ScheduledEvent::macro('notDuringDowntime', fn () => $this->skip(
            function (): bool {
                $status = ServerStatus::query()->latest()->first();

                $should_skip = $status !== null && $status->players === 0;

                if ($should_skip) {
                    info('Skipping scheduled task during downtime (0 players online)');

                    return true;
                }

                return false;
            }
        ));
    }
}
