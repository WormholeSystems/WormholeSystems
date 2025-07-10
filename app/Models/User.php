<?php

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Illuminate\Support\Facades\Session;
use InvalidArgumentException;

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
    use Notifiable;

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
            return $this->getActiveCharacter($this->active_character);
        }
        set {
            $this->active_character = $value;
            if ($value instanceof \App\Models\Character) {
                Session::put(self::SESSION_ACTIVE_CHARACTER_ID, $value->id);
            } else {
                Session::forget(self::SESSION_ACTIVE_CHARACTER_ID);
            }
        }
    }

    public function getActiveCharacterId(): ?int
    {
        return Session::get(self::SESSION_ACTIVE_CHARACTER_ID);
    }

    public function getActiveCharacter(?Character $character = null): ?Character
    {
        if ($character instanceof \App\Models\Character) {
            return $character;
        }

        $active_character_id = $this->getActiveCharacterId();

        if ($active_character_id === null || $active_character_id === 0) {
            $character = $this->characters->first();
            if (! $character) {
                auth()->logout();
                abort(403, 'No characters found. Please log in again.');
            }

            $this->active_character = $character;
        }

        return $this->characters()->find($active_character_id);
    }

    /**
     * @throws InvalidArgumentException
     **/
    public function setActiveCharacter(int|Character|null $character): ?Character
    {
        if ($character === null) {
            Session::forget(self::SESSION_ACTIVE_CHARACTER_ID);

            return null;
        }

        $active_character = $character instanceof Character ? $character : $this->characters()->find($character);

        if (! $active_character) {
            throw new InvalidArgumentException('Character does not belong to this user!');
        }

        Session::put(self::SESSION_ACTIVE_CHARACTER_ID, $active_character->id);

        return $active_character;
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
