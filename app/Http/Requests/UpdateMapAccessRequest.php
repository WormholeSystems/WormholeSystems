<?php

namespace App\Http\Requests;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\MapAccess;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class UpdateMapAccessRequest extends FormRequest
{
    public function __construct(#[RouteParameter('map')] public Map $map)
    {
        parent::__construct();
    }

    public ?MapAccess $map_access {
        get {
            return MapAccess::query()
                ->where('map_id', $this->map->id)
                ->where('accessible_type', sprintf('App\\Models\\%s', $this->string('entity_type')->ucfirst()))
                ->where('accessible_id', $this->integer('entity_id'))
                ->first();
        }
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return $this->user()?->can('update', $this->map) && ! $this->map_access?->is_owner;
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
        ];
    }
}
