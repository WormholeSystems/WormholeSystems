<?php

declare(strict_types=1);

use Illuminate\Database\Migrations\Migration;
use Illuminate\Support\Facades\DB;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        // Link existing signatures to signature_types based on the 'type' column matching 'name'
        DB::statement('
            UPDATE signatures s
            INNER JOIN signature_types st ON s.type = st.name
            SET s.signature_type_id = st.id
            WHERE s.signature_type_id IS NULL
        ');
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        // Unlink signatures from signature_types
        DB::statement('
            UPDATE signatures
            SET signature_type_id = NULL
            WHERE signature_type_id IS NOT NULL
        ');
    }
};
