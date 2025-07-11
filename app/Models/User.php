<?php

namespace App\Models;

use Carbon\CarbonImmutable;
use Database\Factories\UserFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Illuminate\Support\Facades\Session;

/**
 * User model representing a user in the application.
 *
 * @property int $id
 * @property string $name
 * @property Character|null $active_character
 * @property-read Collection<int,Character> $characters
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 */
class User extends Authenticatable
{
    /** @use HasFactory<UserFactory> */
    use HasFactory, Notifiable;

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var list<string>
     */
    protected $hidden = [
    ];

    /**
     * Get the attributes that should be cast.
     *
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
        ];
    }

    protected const string SESSION_ACTIVE_CHARACTER_ID = 'active_character_id';

    public function getAuthPassword(): string
    {
        return '';
    }

    public ?Character $active_character = null {
        get {
            if ($this->active_character instanceof Character) {
                return $this->active_character;
            }

            $active_character_id = Session::get(self::SESSION_ACTIVE_CHARACTER_ID);

            $character = null;
            if (! $active_character_id) {
                $character = $this->characters->first();
                if (! $character) {
                    auth()->logout();
                    abort(403, 'No characters found. Please log in again.');
                }

                $this->active_character = $character;
            }

            return $character ?? $this->characters->find(Session::get(self::SESSION_ACTIVE_CHARACTER_ID));
        }
        set {
            if ($value instanceof Character) {
                Session::put(self::SESSION_ACTIVE_CHARACTER_ID, $value->id);
                $this->active_character = $value;

            } else {
                Session::forget(self::SESSION_ACTIVE_CHARACTER_ID);
                $this->active_character = null;
            }
        }
    }

    public function getAccessibleIds(): array
    {
        return $this->characters->map(fn (Character $character): array => [
            $character->id,
            $character->corporation_id,
            $character->alliance_id,
        ])
            ->flatten()
            ->whereNotNull()
            ->unique()
            ->values()
            ->toArray();
    }

    /**
     * Get the characters associated with the user.
     *
     * @return HasMany<Character, $this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(Character::class, 'user_id');
    }
}
