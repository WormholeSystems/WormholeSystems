<?php

declare(strict_types=1);

namespace Tests;

use Illuminate\Foundation\Testing\TestCase as BaseTestCase;

/**
 * Browser tests exercise the app through a real HTTP server in another
 * process, so the transaction-per-test strategy of the base TestCase does not
 * apply — the data has to actually be committed. Each test gets a freshly
 * migrated database instead, and the real Vite build is served.
 */
abstract class BrowserTestCase extends BaseTestCase
{
    protected function setUp(): void
    {
        parent::setUp();
        $this->artisan('migrate:fresh')->run();
    }
}
