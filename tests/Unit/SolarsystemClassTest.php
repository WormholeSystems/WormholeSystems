<?php

declare(strict_types=1);

use App\Casts\SolarsystemClassCast;
use App\Enums\SolarsystemClass;
use App\Models\WormholeSystem;

it('resolves the known-space class from a security status', function (float $security, SolarsystemClass $expected) {
    expect(SolarsystemClass::fromSecurity($security))->toBe($expected);
})->with([
    'high security' => [0.5, SolarsystemClass::H],
    'rounded high security' => [1.0, SolarsystemClass::H],
    'low security upper' => [0.4, SolarsystemClass::L],
    'low security lower' => [0.1, SolarsystemClass::L],
    'null security' => [0.0, SolarsystemClass::N],
    'negative null security' => [-1.0, SolarsystemClass::N],
]);

it('exposes colour tokens and short labels for rendering', function () {
    expect(SolarsystemClass::C3->colorToken())->toBe('c3')
        ->and(SolarsystemClass::H->colorToken())->toBe('hs')
        ->and(SolarsystemClass::C3->shortLabel())->toBe('C3')
        ->and(SolarsystemClass::H->shortLabel())->toBe('H')
        ->and(SolarsystemClass::Pochven->shortLabel())->toBe('P');
});

it('classifies class groups', function () {
    expect(SolarsystemClass::C3->isStandard())->toBeTrue()
        ->and(SolarsystemClass::C3->isWormholeSpace())->toBeTrue()
        ->and(SolarsystemClass::C14->isDrifter())->toBeTrue()
        ->and(SolarsystemClass::H->isKnownSpace())->toBeTrue()
        ->and(SolarsystemClass::H->isWormholeSpace())->toBeFalse();
});

it('treats abyssal/void classes (C19-C23) as neither wormhole nor known space', function () {
    expect(SolarsystemClass::C19->isWormholeSpace())->toBeFalse()
        ->and(SolarsystemClass::C19->isSpecial())->toBeFalse()
        ->and(SolarsystemClass::C19->isDrifter())->toBeFalse()
        ->and(SolarsystemClass::C23->isWormholeSpace())->toBeFalse()
        ->and(SolarsystemClass::C23->isKnownSpace())->toBeFalse();
});

it('orders known space before wormhole space by sort weight', function () {
    expect(SolarsystemClass::H->sortWeight())->toBeLessThan(SolarsystemClass::C1->sortWeight())
        ->and(SolarsystemClass::C1->sortWeight())->toBeLessThan(SolarsystemClass::C6->sortWeight());
});

it('casts an integer column to and from the enum', function () {
    $cast = new SolarsystemClassCast;
    $model = new WormholeSystem;

    expect($cast->get($model, 'class', 13, []))->toBe(SolarsystemClass::C13)
        ->and($cast->get($model, 'class', '6', []))->toBe(SolarsystemClass::C6)
        ->and($cast->get($model, 'class', null, []))->toBeNull()
        ->and($cast->set($model, 'class', SolarsystemClass::C13, []))->toBe('13')
        ->and($cast->set($model, 'class', 6, []))->toBe('6')
        ->and($cast->set($model, 'class', null, []))->toBeNull();
});

it('reads the class attribute as an enum through the model cast', function () {
    $model = new WormholeSystem;
    $model->setRawAttributes(['class' => 5]);

    expect($model->class)->toBe(SolarsystemClass::C5);
});
