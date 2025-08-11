<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\JsonResponse;

final class PingController extends Controller
{
    public function __construct(#[CurrentUser] private readonly User $user) {}

    public function show(Map $map): JsonResponse
    {
        $this->user->touch('last_active_at');

        return response()->json([
            'message' => 'pong',
            'map_id' => $map->id,
            'user_id' => $this->user->id,
            'last_active_at' => $this->user->last_active_at,
        ]);
    }
}
