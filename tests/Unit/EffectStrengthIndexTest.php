<?php

declare(strict_types=1);

use App\Console\Commands\GenerateStaticDataCommand;

it('maps wormhole classes onto the six effect strength tiers', function (int $class, int $index) {
    expect(GenerateStaticDataCommand::effectStrengthIndex($class))->toBe($index);
})->with([
    'C1' => [1, 0],
    'C2' => [2, 1],
    'C6' => [6, 5],
    'C13 small-ship shattered uses C6 strength' => [13, 5],
    'Sentinel uses C2 strength' => [14, 1],
    'Redoubt uses C2 strength' => [18, 1],
]);
