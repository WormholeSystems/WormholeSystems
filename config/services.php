<?php

declare(strict_types=1);

return [

    /*
    |--------------------------------------------------------------------------
    | Third Party Services
    |--------------------------------------------------------------------------
    |
    | This file is for storing the credentials for third party services such
    | as Mailgun, Postmark, AWS and more. This file provides the de facto
    | location for this type of information, allowing packages to have
    | a conventional file to locate the various service credentials.
    |
    */

    'postmark' => [
        'token' => env('POSTMARK_TOKEN'),
    ],

    'ses' => [
        'key' => env('AWS_ACCESS_KEY_ID'),
        'secret' => env('AWS_SECRET_ACCESS_KEY'),
        'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
    ],

    'resend' => [
        'key' => env('RESEND_KEY'),
    ],

    'slack' => [
        'notifications' => [
            'bot_user_oauth_token' => env('SLACK_BOT_USER_OAUTH_TOKEN'),
            'channel' => env('SLACK_BOT_USER_DEFAULT_CHANNEL'),
        ],
    ],
    'zkillboard' => [
        'identifier' => env('ZKILLBOARD_IDENTIFIER', 'nbrvecs7654vb68mnbv'),

        // Killmails older than this are skipped during ingestion.
        'max_age_days' => env('ZKILLBOARD_MAX_AGE_DAYS', 2 * 365),

        // Base URL for the R2Z2 ephemeral killmail stream.
        'r2z2_base_url' => env('ZKILLBOARD_R2Z2_BASE_URL', 'https://r2z2.zkillboard.com'),

        // Number of HTTP retry attempts before giving up on a request.
        'retry_attempts' => env('ZKILLBOARD_RETRY_ATTEMPTS', 5),

        // Delay between HTTP retries. 6s matches R2Z2's recommended minimum backoff.
        'retry_delay_ms' => env('ZKILLBOARD_RETRY_DELAY_MS', 6_000),

        // Delay between polls when actively consuming new killmails.
        'poll_delay_ms' => env('ZKILLBOARD_POLL_DELAY_MS', 100),

        // Delay when caught up or after an error. 6s matches R2Z2's recommended minimum backoff.
        'catchup_delay_ms' => env('ZKILLBOARD_CATCHUP_DELAY_MS', 6_000),
    ],
    'eveonline' => [
        'client_id' => env('EVE_CLIENT_ID'),
        'client_secret' => env('EVE_CLIENT_SECRET'),
        'redirect' => env('EVE_CALLBACK', 'https://wormholesystems.test/eve/callback'),
    ],
    'discord' => [
        'invite' => env('DISCORD_INVITE', ''),
    ],
];
