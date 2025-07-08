<?php

namespace App\Http\Requests\Signatures;

use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class StoreSignaturesRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return true;
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'signatures' => 'nullable|sometimes|array',
            'signatures.*.signature_id' => 'required|string',
            'signatures.*.type' => 'required|string',
            'signatures.*.category' => 'nullable|string',
            'signatures.*.name' => 'nullable|string',
        ];
    }
}
