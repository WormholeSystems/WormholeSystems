<?php

// config for NicolasKion/Esi
return [
    'user_agent' => env('APP_NAME'),
    'client_id' => env('EVE_CLIENT_ID'),
    'client_secret' => env('EVE_CLIENT_SECRET'),
    'retry_policy' => [
        'tries' => 5,
        'delay' => 5000,
    ],
];
