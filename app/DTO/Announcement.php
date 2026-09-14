<?php

declare(strict_types=1);

namespace App\DTO;

use App\Enums\AnnouncementLevel;
use Illuminate\Contracts\Support\Arrayable;

final class Announcement implements Arrayable
{
    public function __construct(
        public string $message,
        public AnnouncementLevel $level = AnnouncementLevel::Info,
        public ?string $title = null,
        public ?string $link_url = null,
        public ?string $link_label = null,
        public bool $dismissible = true,
    ) {}

    /**
     * The announcement configured for this instance, or null when none is set.
     */
    public static function fromConfig(): ?self
    {
        $message = self::string('announcement.message');

        if ($message === null) {
            return null;
        }

        $link_url = self::string('announcement.link_url');

        return new self(
            message: $message,
            level: AnnouncementLevel::tryFrom((string) config('announcement.level')) ?? AnnouncementLevel::Info,
            title: self::string('announcement.title'),
            link_url: $link_url,
            link_label: $link_url === null ? null : self::string('announcement.link_label') ?? $link_url,
            dismissible: (bool) config('announcement.dismissible', true),
        );
    }

    /**
     * Identifies this exact announcement so a dismissal stops applying as soon
     * as any of its content changes.
     */
    public function id(): string
    {
        return mb_substr(md5(implode('|', [
            $this->level->value,
            $this->title,
            $this->message,
            $this->link_url,
            $this->link_label,
        ])), 0, 12);
    }

    /**
     * @return array{id: string, level: string, title: ?string, message: string, link: ?array{url: string, label: string}, dismissible: bool}
     */
    public function toArray(): array
    {
        return [
            'id' => $this->id(),
            'level' => $this->level->value,
            'title' => $this->title,
            'message' => $this->message,
            'link' => $this->link_url === null ? null : [
                'url' => $this->link_url,
                'label' => $this->link_label ?? $this->link_url,
            ],
            'dismissible' => $this->dismissible,
        ];
    }

    private static function string(string $key): ?string
    {
        $value = mb_trim((string) config($key, ''));

        return $value === '' ? null : $value;
    }
}
