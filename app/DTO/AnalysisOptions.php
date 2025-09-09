<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class AnalysisOptions
{
    public function __construct(
        public int $mapId,
        public int $daysAgo,
        public int $daysActive,
        public int $top,
        public int $activeThreshold,
        public int $hostileThreshold,
    ) {}

    public static function fromCommand(array $arguments, array $options): self
    {
        return new self(
            mapId: (int) $arguments['map'],
            daysAgo: (int) $options['days-ago'],
            daysActive: (int) $options['days-active'],
            top: (int) $options['top'],
            activeThreshold: (int) $options['active-threshold'],
            hostileThreshold: (int) $options['hostile-threshold'],
        );
    }
}
