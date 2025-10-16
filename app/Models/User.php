<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Database\Factories\UserFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Illuminate\Support\Facades\Session;
use Laravel\Sanctum\HasApiTokens;

/**
 * User model representing a user in the application.
 *
 * @property int $id
 * @property string $name
 * @property int|null $preferred_character_id
 * @property Character|null $active_character
 * @property-read Character|null $preferredCharacter
 * @property-read Collection<int,Character> $characters
 * @property-read Collection<int,MapUserSetting> $mapUserSettings
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 */
final class User extends Authenticatable
{
    /** @use HasFactory<UserFactory> */
    use HasApiTokens, HasFactory, Notifiable;

    protected const string SESSION_ACTIVE_CHARACTER_ID = 'active_character_id';

    public ?Character $active_character = null {
        get {
            if ($this->active_character instanceof Character) {
                return $this->active_character;
            }

            // First, check session for active character
            $active_character_id = Session::get(self::SESSION_ACTIVE_CHARACTER_ID);

            // If no session, use the preferred character from database
            if (! $active_character_id && $this->preferred_character_id) {
                $active_character_id = $this->preferred_character_id;
            }

            $character = null;
            if (! $active_character_id) {
                // Fall back to first character
                $character = $this->characters->first();
                if (! $character) {
                    auth()->logout();
                    abort(403, 'No characters found. Please log in again.');
                }

                $this->active_character = $character;
            }

            return $character ?? $this->characters->find($active_character_id);
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

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var list<string>
     */
    protected $hidden = [
    ];

    public function getAuthPassword(): string
    {
        return '';
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
     * Get the preferred character for the user.
     *
     * @return BelongsTo<Character, $this>
     */
    public function preferredCharacter(): BelongsTo
    {
        return $this->belongsTo(Character::class, 'preferred_character_id');
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

    /**
     * Get the user settings for the map.
     *
     * @return HasMany<MapUserSetting, $this>
     */
    public function mapUserSettings(): HasMany
    {
        return $this->hasMany(MapUserSetting::class, 'user_id');
    }

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
}
