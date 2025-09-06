<?php

declare(strict_types=1);

use App\Models\Signature;
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
        Schema::table('signatures', function (Blueprint $table): void {
            $table->dateTime('marked_as_eol_at')->nullable()->after('is_eol');
        });

        Signature::query()
            ->where('is_eol', true)
            ->update(['marked_as_eol_at' => now()]);

        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropColumn('is_eol');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('signatures', function (Blueprint $table): void {
            $table->boolean('is_eol')->default(false)->after('marked_as_eol_at');
        });

        Signature::query()
            ->whereNotNull('marked_as_eol_at')
            ->update(['is_eol' => true]);

        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropColumn('marked_as_eol_at');
        });
    }
};
