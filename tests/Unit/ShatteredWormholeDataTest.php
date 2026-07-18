<?php

declare(strict_types=1);

it('flags exactly the shattered wormhole constellations in the source data', function () {
    $handle = fopen(__DIR__.'/../../database/seeders/data/wormhole_systems.csv', 'rb');
    $header = fgetcsv($handle, escape: '\\');
    $index = array_flip($header);

    $shattered_count = 0;

    while (($row = fgetcsv($handle, escape: '\\')) !== false) {
        $name = (string) $row[$index['name']];
        $constellation_id = (int) $row[$index['constellation_id']];
        $is_shattered = mb_trim((string) $row[$index['shattered']]) !== '';

        // The shattered systems occupy their own constellations, verified
        // against the community wormhole list.
        $expected = $constellation_id >= 21_000_324 && $constellation_id <= 21_000_334;

        expect($is_shattered)->toBe($expected, "{$name} shattered flag does not match its constellation");

        if ($is_shattered) {
            $shattered_count++;
        }
    }

    fclose($handle);

    // Thera + 75 standard-class + 25 frigate C13s + 5 drifter systems.
    expect($shattered_count)->toBe(106);
});
