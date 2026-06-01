<?php

declare(strict_types=1);

/*
|--------------------------------------------------------------------------
| Test Case
|--------------------------------------------------------------------------
|
| The closure you provide to your test functions is always bound to a specific PHPUnit test
| case class. By default, that class is "PHPUnit\Framework\TestCase". Of course, you may
| need to change it using the "pest()" function to bind a different classes or traits.
|
*/

pest()->extend(Tests\TestCase::class)
    ->in('Feature');
/*
|--------------------------------------------------------------------------
| Expectations
|--------------------------------------------------------------------------
|
| When you're writing tests, you often need to check that values meet certain conditions. The
| "expect()" function gives you access to a set of "expectations" methods that you can use
| to assert different things. Of course, you may extend the Expectation API at any time.
|
*/

beforeEach();

/*
|--------------------------------------------------------------------------
| Functions
|--------------------------------------------------------------------------
|
| While Pest is very powerful out-of-the-box, you may have some testing code specific to your
| project that you don't want to repeat in every file. Here you can also expose helpers as
| global functions to help you to reduce the number of lines of code in your test files.
|
*/

function something()
{
    // ..
}

/**
 * Insert a single solar system (and its parent region/constellation) into the
 * otherwise-empty test database, returning its id. The SDE is not seeded in tests.
 */
function makeSolarsystem(int $id, float $security = 0.5): int
{
    Illuminate\Support\Facades\DB::table('regions')->insertOrIgnore(['id' => 10009000, 'name' => 'Test Region', 'type' => 'normal']);
    Illuminate\Support\Facades\DB::table('constellations')->insertOrIgnore(['id' => 20009000, 'name' => 'Test Constellation', 'region_id' => 10009000, 'type' => 'normal']);
    Illuminate\Support\Facades\DB::table('solarsystems')->insertOrIgnore([
        'id' => $id,
        'name' => 'Test System '.$id,
        'constellation_id' => 20009000,
        'region_id' => 10009000,
        'security' => $security,
        'pos_x' => 0,
        'pos_y' => 0,
        'pos_z' => 0,
        'type' => 'normal',
    ]);

    return $id;
}
