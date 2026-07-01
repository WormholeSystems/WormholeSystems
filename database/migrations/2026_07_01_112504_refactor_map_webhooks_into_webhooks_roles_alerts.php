<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Split the bundled `map_webhooks` table into reusable webhook destinations,
     * reusable roles, and the alerts that reference them.
     */
    public function up(): void
    {
        Schema::create('map_webhook_roles', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained()->cascadeOnDelete();
            $table->string('name');
            $table->string('discord_role_id');
            $table->timestamps();

            $table->unique(['map_id', 'discord_role_id']);
        });

        Schema::create('map_alerts', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_id')->constrained()->cascadeOnDelete();
            $table->foreignId('map_webhook_id')->constrained()->cascadeOnDelete();
            $table->foreignId('map_webhook_role_id')->nullable()->constrained()->nullOnDelete();
            $table->string('type')->default('proximity')->index();
            $table->foreignId('target_solarsystem_id')->nullable()->constrained('solarsystems')->cascadeOnDelete();
            $table->unsignedTinyInteger('max_jumps');
            $table->json('filters')->nullable();
            $table->string('filter_match')->default('any');
            $table->boolean('is_active')->default(true)->index();
            $table->timestamp('last_fired_at')->nullable();
            $table->timestamps();
        });

        $this->splitExistingWebhooks();

        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropForeign(['target_solarsystem_id']);
        });

        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->dropColumn([
                'type',
                'target_solarsystem_id',
                'max_jumps',
                'filters',
                'filter_match',
                'discord_role_id',
                'is_active',
                'last_fired_at',
            ]);
        });
    }

    public function down(): void
    {
        Schema::table('map_webhooks', function (Blueprint $table): void {
            $table->string('type')->default('proximity')->index();
            $table->foreignId('target_solarsystem_id')->nullable()->constrained('solarsystems')->cascadeOnDelete();
            $table->unsignedTinyInteger('max_jumps')->default(5);
            $table->json('filters')->nullable();
            $table->string('filter_match')->default('any');
            $table->string('discord_role_id')->nullable();
            $table->boolean('is_active')->default(true)->index();
            $table->timestamp('last_fired_at')->nullable();
        });

        Schema::dropIfExists('map_alerts');
        Schema::dropIfExists('map_webhook_roles');
    }

    /**
     * Turn each existing bundled webhook into a webhook destination that keeps its
     * name + url, an optional deduplicated role, and an alert carrying the trigger.
     */
    private function splitExistingWebhooks(): void
    {
        $roleIds = [];

        DB::table('map_webhooks')->orderBy('id')->each(function (object $webhook) use (&$roleIds): void {
            $roleId = null;

            if ($webhook->discord_role_id !== null && $webhook->discord_role_id !== '') {
                $key = $webhook->map_id.':'.$webhook->discord_role_id;

                $roleId = $roleIds[$key] ??= DB::table('map_webhook_roles')->insertGetId([
                    'map_id' => $webhook->map_id,
                    'name' => 'Role '.mb_substr($webhook->discord_role_id, -4),
                    'discord_role_id' => $webhook->discord_role_id,
                    'created_at' => $webhook->created_at,
                    'updated_at' => $webhook->updated_at,
                ]);
            }

            DB::table('map_alerts')->insert([
                'map_id' => $webhook->map_id,
                'map_webhook_id' => $webhook->id,
                'map_webhook_role_id' => $roleId,
                'type' => $webhook->type,
                'target_solarsystem_id' => $webhook->target_solarsystem_id,
                'max_jumps' => $webhook->max_jumps,
                'filters' => $webhook->filters,
                'filter_match' => $webhook->filter_match,
                'is_active' => $webhook->is_active,
                'last_fired_at' => $webhook->last_fired_at,
                'created_at' => $webhook->created_at,
                'updated_at' => $webhook->updated_at,
            ]);
        });
    }
};
