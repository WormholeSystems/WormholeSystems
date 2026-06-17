<?php

declare(strict_types=1);

return [
    /*
    |--------------------------------------------------------------------------
    | Documentation edit base URL
    |--------------------------------------------------------------------------
    |
    | The base URL used to build the "Edit this page on GitHub" link for each
    | documentation page. The page's repository-relative path is appended to
    | this value, e.g. ".../edit/main/resources/docs/getting-started/overview.md".
    |
    */

    'edit_base_url' => env('DOCS_EDIT_BASE_URL', 'https://github.com/WormholeSystems/WormholeSystems/edit/main'),
];
