<?php

declare(strict_types=1);

use App\Enums\ShipSize;
use App\Models\MapConnection;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Null becomes the "nobody has said yet" size. Existing rows keep the large
     * they were given by the old default: there is no way to tell now which of
     * them were deliberate, and blanking them would throw away real answers.
     */
    public function up(): void
    {
        Schema::table('map_connections', function (Blueprint $table): void {
            $table->string('ship_size')->nullable()->change();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        MapConnection::query()->whereNull('ship_size')->update(['ship_size' => ShipSize::Large->value]);

        Schema::table('map_connections', function (Blueprint $table): void {
            $table->string('ship_size')->default(ShipSize::Large->value)->change();
        });
    }
};
