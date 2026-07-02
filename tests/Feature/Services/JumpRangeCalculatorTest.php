<?php

declare(strict_types=1);

use App\Enums\JumpShipType;
use App\Models\Solarsystem;
use App\Services\JumpRange\JumpRangeCalculator;

it('computes the euclidean distance in light years', function () {
    $calculator = new JumpRangeCalculator;

    $from = new Solarsystem(['pos_x' => 0.0, 'pos_y' => 0.0, 'pos_z' => 0.0]);
    $to = new Solarsystem([
        'pos_x' => 3.0 * JumpRangeCalculator::METERS_PER_LIGHTYEAR,
        'pos_y' => 4.0 * JumpRangeCalculator::METERS_PER_LIGHTYEAR,
        'pos_z' => 0.0,
    ]);

    expect($calculator->distanceLy($from, $to))->toEqualWithDelta(5.0, 0.0001);
});

it('scales jump range with the JDC level', function (JumpShipType $ship, int $jdcLevel, float $expected) {
    expect($ship->maxRangeLy($jdcLevel))->toEqualWithDelta($expected, 0.0001);
})->with([
    'dreadnought JDC 5' => [JumpShipType::Dreadnought, 5, 7.0],
    'dreadnought JDC 1' => [JumpShipType::Dreadnought, 1, 4.2],
    'titan JDC 5' => [JumpShipType::Titan, 5, 6.0],
    'jump freighter JDC 5' => [JumpShipType::JumpFreighter, 5, 10.0],
    'black ops JDC 4' => [JumpShipType::BlackOps, 4, 7.2],
]);
