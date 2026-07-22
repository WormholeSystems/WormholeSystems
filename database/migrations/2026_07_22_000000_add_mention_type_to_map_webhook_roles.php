<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('map_webhook_roles', function (Blueprint $table): void {
            $table->string('mention_type')->default('role')->after('name');
        });
    }

    public function down(): void
    {
        Schema::table('map_webhook_roles', function (Blueprint $table): void {
            $table->dropColumn('mention_type');
        });
    }
};
