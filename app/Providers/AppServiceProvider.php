<?php

declare(strict_types=1);

namespace App\Providers;

use App\Models\ServerStatus;
use App\Policies\PersonalAccessTokenPolicy;
use App\Services\RouteService;
use Artisan;
use Illuminate\Console\Scheduling\Event as ScheduledEvent;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Event;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Facades\ParallelTesting;
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

        Model::automaticallyEagerLoadRelationships();

        Event::listen(function (SocialiteWasCalled $event): void {
            $event->extendSocialite('eveonline', Provider::class);
        });

        ParallelTesting::setUpTestDatabase(function (): void {
            Artisan::call('db:restore', [
                '--database' => 'test_database',
            ]);
        });

        Gate::policy(PersonalAccessToken::class, PersonalAccessTokenPolicy::class);

        $this->registerScheduleMacros();
    }

    private function registerNotificationMacro(): void
    {
        RedirectResponse::macro('notify', function (string $title, string $message = '', string $type = 'success') {
            if (request()->boolean('silent')) {
                return $this;
            }

            return $this->with('notification', [
                'title' => $title,
                'message' => $message,
                'type' => $type,
            ]);
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
