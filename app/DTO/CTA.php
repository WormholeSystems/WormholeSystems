<?php

declare(strict_types=1);

namespace App\DTO;

use Illuminate\Contracts\Support\Arrayable;

final readonly class CTA implements Arrayable
{
    public function __construct(
        public string $title = '',
        public string $url = '',
        public bool $external = false,
    ) {}

    public static function make(): self
    {
        return new self();
    }

    public function toArray(): array
    {
        return [
            'title' => $this->title,
            'url' => $this->url,
            'external' => $this->external,
        ];
    }

    public function title(string $title): self
    {
        return new self(
            title: $title,
            url: $this->url,
            external: $this->external,
        );
    }

    public function url(string $url): self
    {
        return new self(
            title: $this->title,
            url: $url,
            external: $this->external,
        );
    }

    public function external(bool $external = true): self
    {
        return new self(
            title: $this->title,
            url: $this->url,
            external: $external,
        );
    }
}
