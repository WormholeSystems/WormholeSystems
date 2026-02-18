<?php

declare(strict_types=1);

use App\Enums\Permission;
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
        Schema::create('map_access', function (Blueprint $table): void {
            $table->id();
            $table->morphs('accessible');
            $table->foreignId('map_id')->constrained('maps')->onDelete('cascade');
            $table->string('permission')->default(Permission::Manager);
            $table->boolean('is_owner')->default(false);
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('map_access');
    }
};
