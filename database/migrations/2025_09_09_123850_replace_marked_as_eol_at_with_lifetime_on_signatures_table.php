<?php

declare(strict_types=1);

use App\Enums\LifetimeStatus;
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
        Schema::table('signatures', function (Blueprint $table): void {
            // Add the new lifetime columns
            $table->string('lifetime')->default(LifetimeStatus::Healthy->value)->after('marked_as_eol_at');
            $table->dateTime('lifetime_updated_at')->nullable()->after('lifetime');
        });

        // Migrate existing data: marked_as_eol_at means EOL (End of Life)
        DB::table('signatures')
            ->whereNotNull('marked_as_eol_at')
            ->update([
                'lifetime' => LifetimeStatus::EndOfLife->value,
                'lifetime_updated_at' => DB::raw('marked_as_eol_at'),
            ]);

        Schema::table('signatures', function (Blueprint $table): void {
            // Drop the old column and add index
            $table->dropColumn('marked_as_eol_at');
            $table->index('lifetime');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('signatures', function (Blueprint $table): void {
            // Add back the old column
            $table->dateTime('marked_as_eol_at')->nullable()->after('lifetime');
        });

        // Migrate data back (only mark as EOL if not healthy)
        DB::table('signatures')
            ->where('lifetime', '!=', LifetimeStatus::Healthy->value)
            ->update(['marked_as_eol_at' => DB::raw('NOW()')]);

        Schema::table('signatures', function (Blueprint $table): void {
            // Drop the new columns
            $table->dropIndex(['lifetime']);
            $table->dropColumn(['lifetime', 'lifetime_updated_at']);
        });
    }
};
