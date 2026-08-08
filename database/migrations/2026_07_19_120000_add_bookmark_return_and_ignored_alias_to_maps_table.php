<?php

declare(strict_types=1);

use App\Enums\BookmarkToken;
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
            $table->string('bookmark_ignored_alias')->default(BookmarkToken::DEFAULT_IGNORED_ALIAS);
            $table->string('bookmark_format_return')->default(BookmarkToken::DEFAULT_RETURN);
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::table('maps', function (Blueprint $table): void {
            $table->dropColumn(['bookmark_ignored_alias', 'bookmark_format_return']);
        });
    }
};
