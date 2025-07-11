<?php

namespace App\Providers;

use Artisan;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Event;
use Illuminate\Support\Facades\ParallelTesting;
use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        //
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        JsonResource::withoutWrapping();

        $this->registerNotificationMacro();

        Event::listen(function (\SocialiteProviders\Manager\SocialiteWasCalled $event): void {
            $event->extendSocialite('eveonline', \SocialiteProviders\Eveonline\Provider::class);
        });

        // Executed when a test database is created...
        ParallelTesting::setUpTestDatabase(function (string $database, int $token) {
            Artisan::call('db:restore', [
                '--database' => 'test_database',
            ]);
        });
    }

    protected function registerNotificationMacro(): void
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
