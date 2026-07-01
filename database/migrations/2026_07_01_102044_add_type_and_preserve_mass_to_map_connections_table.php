<?php

declare(strict_types=1);

use App\Enums\ConnectionType;
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
        Schema::table('map_connections', function (Blueprint $table): void {
            $table->string('type')->default(ConnectionType::Wormhole->value)->index();
            $table->boolean('preserve_mass')->default(false);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('map_connections', function (Blueprint $table): void {
            $table->dropColumn(['type', 'preserve_mass']);
        });
    }
};
