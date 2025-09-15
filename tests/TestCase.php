<?php

declare(strict_types=1);

namespace Tests;

use Illuminate\Foundation\Testing\TestCase as BaseTestCase;

abstract class TestCase extends BaseTestCase
{
    protected function setUp(): void
    {
        parent::setUp();
        $this->artisan('migrate:fresh')->run();

        if ($this->artisan('db:restore')->execute()) {
            $this->artisan('db:seed')->run();
            $this->artisan('db:cache')->run();
        }
    }
}
