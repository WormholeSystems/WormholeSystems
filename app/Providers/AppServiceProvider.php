<?php

declare(strict_types=1);

namespace App\Providers;

use App\Services\RouteService;
use Artisan;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Event;
use Illuminate\Support\Facades\ParallelTesting;
use Illuminate\Support\ServiceProvider;
use SocialiteProviders\Eveonline\Provider;
use SocialiteProviders\Manager\SocialiteWasCalled;

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
}
