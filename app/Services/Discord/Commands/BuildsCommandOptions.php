<?php

declare(strict_types=1);

namespace App\Services\Discord\Commands;

trait BuildsCommandOptions
{
    private function mapOption(): Option
    {
        return Option::string('map')->description('Wormhole Systems map')->required()->autocomplete();
    }

    private function systemOption(): Option
    {
        return Option::string('system')->description('EVE solar system')->required()->autocomplete();
    }

    private function jumpsOption(): Option
    {
        return Option::integer('jumps')->description('Maximum gate jumps')->required()->between(1, 20);
    }

    private function alertOption(): Option
    {
        return Option::string('alert')->description('Alert')->required()->autocomplete();
    }

    private function fromOption(): Option
    {
        return Option::string('from')->description('Starting point system (defaults to the chain)')->autocomplete();
    }

    private function mentionOption(): Option
    {
        return Option::string('mention')
            ->description('Who to ping')
            ->required()
            ->choice('Nobody', 'none')
            ->choice('Me', 'creator')
            ->choice('A role', 'role')
            ->choice('Everyone', 'everyone');
    }

    private function roleOption(): Option
    {
        return Option::role('role')->description('Role to ping');
    }

    /** @return Option[] */
    private function mentionOptions(): array
    {
        return [$this->mentionOption(), $this->roleOption()];
    }
}
