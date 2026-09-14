<?php

declare(strict_types=1);

namespace App\DTO;

final readonly class MaintainerCharacterStat
{
    public function __construct(
        public int $character_id,
        public string $character_name,
        public ?int $user_id,
        public int $nb_added,
        public int $nb_edited,
        public int $nb_deleted,
        public int $points,
    ) {}

    /**
     * @param  array{character_id: int, character_name: string, user_id: int|null, nb_added: int, nb_edited: int, nb_deleted: int, points: int}  $data
     */
    public static function fromArray(array $data): self
    {
        return new self(
            character_id: $data['character_id'],
            character_name: $data['character_name'],
            user_id: $data['user_id'],
            nb_added: $data['nb_added'],
            nb_edited: $data['nb_edited'],
            nb_deleted: $data['nb_deleted'],
            points: $data['points'],
        );
    }

    /**
     * @return array{character_id: int, character_name: string, user_id: int|null, nb_added: int, nb_edited: int, nb_deleted: int, points: int}
     */
    public function toArray(): array
    {
        return [
            'character_id' => $this->character_id,
            'character_name' => $this->character_name,
            'user_id' => $this->user_id,
            'nb_added' => $this->nb_added,
            'nb_edited' => $this->nb_edited,
            'nb_deleted' => $this->nb_deleted,
            'points' => $this->points,
        ];
    }
}
