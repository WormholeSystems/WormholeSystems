<?php

declare(strict_types=1);

return [
    /*
    |--------------------------------------------------------------------------
    | Affiliation Login Whitelist
    |--------------------------------------------------------------------------
    |
    | A whitelist of EVE Online affiliation IDs (character, corporation, or
    | alliance) that are allowed to log in to this instance. A character may
    | log in when any of its affiliations is present in this list.
    |
    | Leave empty to allow anyone to log in (the default).
    |
    */

    'allowed_affiliation_ids' => array_values(array_filter(array_map(
        static fn (string $id): int => (int) mb_trim($id),
        explode(',', (string) env('ALLOWED_AFFILIATION_IDS', '')),
    ))),
];
