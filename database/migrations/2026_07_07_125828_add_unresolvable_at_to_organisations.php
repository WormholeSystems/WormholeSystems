<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /** Marks organisations ESI answers 404 for, so the name sweep stops retrying them. */
    public function up(): void
    {
        Schema::table('corporations', function (Blueprint $table): void {
            $table->timestamp('unresolvable_at')->nullable()->after('last_updated');
        });

        Schema::table('alliances', function (Blueprint $table): void {
            $table->timestamp('unresolvable_at')->nullable()->after('last_updated');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('corporations', function (Blueprint $table): void {
            $table->dropColumn('unresolvable_at');
        });

        Schema::table('alliances', function (Blueprint $table): void {
            $table->dropColumn('unresolvable_at');
        });
    }
};
