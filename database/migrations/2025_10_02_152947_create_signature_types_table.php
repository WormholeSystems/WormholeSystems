<?php

declare(strict_types=1);

use Database\Seeders\SignatureTypeSeeder;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Artisan;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('signature_types', function (Blueprint $table) {
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

        Artisan::call('db:seed', ['--class' => SignatureTypeSeeder::class]);
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('signature_types');
    }
};
