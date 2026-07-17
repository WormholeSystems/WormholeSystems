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

pest()->extend(Tests\BrowserTestCase::class)
    ->in('Browser');

pest()->use(Tests\Browser\Concerns\InteractsWithMap::class)
    ->in('Browser');
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
function makeSolarsystem(int $id, float $security = 0.5, string $type = 'normal', float $posX = 0.0, float $posY = 0.0, float $posZ = 0.0): int
{
    Illuminate\Support\Facades\DB::table('regions')->insertOrIgnore(['id' => 10009000, 'name' => 'Test Region', 'type' => 'normal']);
    Illuminate\Support\Facades\DB::table('constellations')->insertOrIgnore(['id' => 20009000, 'name' => 'Test Constellation', 'region_id' => 10009000, 'type' => 'normal']);
    Illuminate\Support\Facades\DB::table('solarsystems')->insertOrIgnore([
        'id' => $id,
        'name' => 'Test System '.$id,
        'constellation_id' => 20009000,
        'region_id' => 10009000,
        'security' => $security,
        'pos_x' => $posX,
        'pos_y' => $posY,
        'pos_z' => $posZ,
        'type' => $type,
    ]);

    return $id;
}

/**
 * Place a solarsystem on a map (seeding the underlying solarsystem so the FK resolves),
 * returning the placement.
 */
/** Create a wormhole type row; defaults to a capital-capable H296. */
function makeWormhole(string $name = 'H296', float $maximum_jump_mass = 2_000_000_000, string $leads_to = 'c5'): App\Models\Wormhole
{
    return App\Models\Wormhole::create([
        'name' => $name,
        'total_mass' => 3_300_000_000,
        'maximum_jump_mass' => $maximum_jump_mass,
        'maximum_lifetime' => 86_400,
        'leads_to' => $leads_to,
    ]);
}

function placeMapSolarsystem(App\Models\Map $map, int $solarsystemId, int $x = 100, int $y = 100): App\Models\MapSolarsystem
{
    makeSolarsystem($solarsystemId);

    return App\Models\MapSolarsystem::factory()->for($map)->create([
        'solarsystem_id' => $solarsystemId,
        'position_x' => $x,
        'position_y' => $y,
        'pinned' => false,
    ]);
}
