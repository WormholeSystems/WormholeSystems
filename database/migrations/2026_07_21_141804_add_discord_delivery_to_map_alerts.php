<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->dropForeign(['map_webhook_id']);
        });

        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->foreignId('created_by_user_id')->nullable()->after('map_id')->constrained('users')->nullOnDelete();
            $table->string('delivery_type')->default('webhook')->after('created_by_user_id')->index();
            $table->foreignId('map_webhook_id')->nullable()->change();
            $table->string('mention_mode')->default('none')->after('map_webhook_role_id');
            $table->string('discord_guild_id')->nullable()->after('mention_mode');
            $table->string('discord_channel_id')->nullable()->after('discord_guild_id');
            $table->string('discord_role_id')->nullable()->after('discord_channel_id');
            $table->timestamp('disabled_at')->nullable()->after('last_fired_at');
            $table->foreignId('disabled_by_user_id')->nullable()->after('disabled_at')->constrained('users')->nullOnDelete();
            $table->string('disabled_reason')->nullable()->after('disabled_by_user_id');
            $table->softDeletes();
            $table->foreignId('deleted_by_user_id')->nullable()->after('deleted_at')->constrained('users')->nullOnDelete();

            $table->foreign('map_webhook_id')->references('id')->on('map_webhooks')->cascadeOnDelete();
        });

        DB::table('map_alerts')->whereNotNull('map_webhook_role_id')->update(['mention_mode' => 'role']);

        Schema::create('map_alert_deliveries', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_alert_id')->constrained()->cascadeOnDelete();
            $table->foreignId('map_solarsystem_id')->constrained()->cascadeOnDelete();
            $table->timestamp('delivered_at')->nullable();
            $table->timestamps();

            $table->unique(['map_alert_id', 'map_solarsystem_id']);
        });

        Schema::create('map_alert_events', function (Blueprint $table): void {
            $table->id();
            $table->foreignId('map_alert_id')->nullable()->constrained()->nullOnDelete();
            $table->unsignedBigInteger('map_id')->index();
            $table->foreignId('actor_user_id')->nullable()->constrained('users')->nullOnDelete();
            $table->string('actor_name')->nullable();
            $table->string('action')->index();
            $table->json('snapshot');
            $table->text('reason')->nullable();
            $table->timestamp('created_at')->useCurrent();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_alert_events');
        Schema::dropIfExists('map_alert_deliveries');

        DB::table('map_alerts')->where('delivery_type', '!=', 'webhook')->delete();
        DB::table('map_alerts')->whereNotNull('deleted_at')->delete();

        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->dropForeign(['created_by_user_id']);
            $table->dropForeign(['disabled_by_user_id']);
            $table->dropForeign(['deleted_by_user_id']);
            $table->dropForeign(['map_webhook_id']);
            $table->dropIndex(['delivery_type']);
            $table->dropColumn([
                'created_by_user_id',
                'delivery_type',
                'mention_mode',
                'discord_guild_id',
                'discord_channel_id',
                'discord_role_id',
                'disabled_at',
                'disabled_by_user_id',
                'disabled_reason',
                'deleted_at',
                'deleted_by_user_id',
            ]);
        });

        Schema::table('map_alerts', function (Blueprint $table): void {
            $table->foreignId('map_webhook_id')->nullable(false)->change();
            $table->foreign('map_webhook_id')->references('id')->on('map_webhooks')->cascadeOnDelete();
        });
    }
};
