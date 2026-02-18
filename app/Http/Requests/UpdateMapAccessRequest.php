<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\Permission;
use App\Models\Alliance;
use App\Models\Character;
use App\Models\Corporation;
use App\Models\Map;
use App\Models\MapAccess;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class UpdateMapAccessRequest extends FormRequest
{
    public ?MapAccess $map_access {
        get => MapAccess::query()
            ->where('map_id', $this->map->id)
            ->where('accessible_type', sprintf('App\\Models\\%s', $this->string('entity_type')->ucfirst()))
            ->where('accessible_id', $this->integer('entity_id'))
            ->first();

    }

    public Alliance|Corporation|Character|null $accessor {
        get => match ($this->string('entity_type')->value()) {
            'character' => Character::find($this->integer('entity_id')),
            'corporation' => Corporation::find($this->integer('entity_id')),
            'alliance' => Alliance::find($this->integer('entity_id')),
            default => null,
        };
    }

    public ?Permission $permission {
        get => $this->enum('permission', Permission::class);
    }

    public function __construct(#[RouteParameter('map')] public Map $map)
    {
        parent::__construct();
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return $this->user()?->can('manageAccess', $this->map) && ! $this->map_access?->is_owner;
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'entity_id' => 'required|integer',
            'entity_type' => 'required|string|in:character,corporation,alliance',
            'permission' => ['nullable', 'sometimes', Rule::enum(Permission::class)],
            'expires_at' => ['nullable', 'date', 'after:now'],
        ];
    }
}
