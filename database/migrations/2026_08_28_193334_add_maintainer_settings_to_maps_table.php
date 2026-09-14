<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('maps', function (Blueprint $table): void {
            $table->unsignedTinyInteger('maintainer_points_created')->default(1);
            $table->unsignedTinyInteger('maintainer_points_updated')->default(1);
            $table->unsignedTinyInteger('maintainer_points_deleted')->default(1);
            $table->unsignedSmallInteger('maintainer_minimum_points')->default(20);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('maps', function (Blueprint $table): void {
            $table->dropColumn([
                'maintainer_points_created',
                'maintainer_points_updated',
                'maintainer_points_deleted',
                'maintainer_minimum_points',
            ]);
        });
    }
};
