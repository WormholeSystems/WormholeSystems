<?php

declare(strict_types=1);

return [
    /*
    |--------------------------------------------------------------------------
    | Site-wide Announcement
    |--------------------------------------------------------------------------
    |
    | A banner shown on every page, used to tell users about planned downtime,
    | migrations or anything else they should read before using the app. It is
    | disabled whenever the message is empty.
    |
    | The level drives the banner's colour: "info", "warning" or "critical".
    | Dismissals are remembered per announcement, so changing any of the values
    | below brings the banner back for everyone.
    |
    */

    'level' => env('ANNOUNCEMENT_LEVEL', 'info'),
    'title' => env('ANNOUNCEMENT_TITLE'),
    'message' => env('ANNOUNCEMENT_MESSAGE'),
    'link_url' => env('ANNOUNCEMENT_LINK_URL'),
    'link_label' => env('ANNOUNCEMENT_LINK_LABEL'),
    'dismissible' => env('ANNOUNCEMENT_DISMISSIBLE', true),
];
