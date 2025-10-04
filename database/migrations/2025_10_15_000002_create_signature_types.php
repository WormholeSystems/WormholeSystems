<?php

declare(strict_types=1);

use Database\Seeders\SignatureTypeSeeder;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Create signature_types table
        Schema::create('signature_types', function (Blueprint $table): void {
            $table->id();
            $table->string('name');
            $table->string('signature')->nullable();
            $table->foreignId('signature_category_id')->constrained('signature_categories')->onDelete('cascade');
            $table->string('target_class')->nullable();
            $table->string('extra')->nullable();
            $table->json('spawn_areas')->nullable();
            $table->timestamps();

            $table->index('signature_category_id');
            $table->index('signature');
            $table->index('target_class');
            $table->unique(['name', 'signature_category_id']);
        });

        // Seed signature_types
        Artisan::call('db:seed', ['--class' => SignatureTypeSeeder::class]);

        // Add signature_type_id to signatures table
        Schema::table('signatures', function (Blueprint $table): void {
            $table->foreignId('signature_type_id')
                ->nullable()
                ->after('signature_category_id')
                ->constrained('signature_types')
                ->onDelete('set null');
        });

        // Link existing signatures to signature_types based on the 'type' column matching 'name'
        DB::statement('
            UPDATE signatures s
            INNER JOIN signature_types st ON s.type = st.name
            SET s.signature_type_id = st.id
            WHERE s.signature_type_id IS NULL
        ');

        // Remove old type column
        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropColumn('type');
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // Restore old type column
        Schema::table('signatures', function (Blueprint $table): void {
            $table->string('type')->nullable()->after('signature_id');
        });

        // Restore type data from signature_types name
        DB::statement('
            UPDATE signatures s
            INNER JOIN signature_types st ON s.signature_type_id = st.id
            SET s.type = st.name
            WHERE s.type IS NULL
        ');

        // Remove signature_type_id
        Schema::table('signatures', function (Blueprint $table): void {
            $table->dropForeign(['signature_type_id']);
            $table->dropColumn('signature_type_id');
        });

        // Drop signature_types table
        Schema::dropIfExists('signature_types');
    }
};
