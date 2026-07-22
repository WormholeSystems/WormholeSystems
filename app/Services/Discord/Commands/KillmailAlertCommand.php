<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

final readonly class KillmailAlertCommand implements AlertVariantDefinition
{
    use BuildsCommandOptions;

    public function definition(bool $withMentions): SubCommand
    {
        $subCommand = SubCommand::make('killmail', 'A kill happens within jumps of the map chain')->options(
            $this->mapOption(),
            $this->jumpsOption(),
        );

        return $withMentions ? $subCommand->options(...$this->mentionOptions()) : $subCommand;
    }
}
