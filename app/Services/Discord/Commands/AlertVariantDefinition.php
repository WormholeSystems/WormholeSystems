<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

interface AlertVariantDefinition
{
    /**
     * The subcommand for this alert type; channel variants additionally carry
     * the mention options.
     */
    public function definition(bool $withMentions): SubCommand;
}
