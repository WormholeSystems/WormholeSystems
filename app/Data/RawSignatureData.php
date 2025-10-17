<?php

declare(strict_types=1);

namespace App\Data;

use Spatie\LaravelData\Attributes\Validation\Exists;
use Spatie\LaravelData\Attributes\Validation\Max;
use Spatie\LaravelData\Attributes\Validation\Min;
use Spatie\LaravelData\Data;
use Spatie\LaravelData\Optional;

final class RawSignatureData extends Data
{
    public function __construct(
        #[Min(7), Max(7)]
        public string $signature_id,
        #[Exists(table: 'signature_types', column: 'id')]
        public ?int $signature_type_id = null,
        #[Exists(table: 'signature_categories', column: 'id')]
        public ?int $signature_category_id = null,
        public null|Optional|string $raw_type_name = null,
    ) {}

}
