<?php

declare(strict_types=1);

namespace App\Actions\MapTransfer;

use App\Enums\AliasScheme;
use App\Enums\ConnectionType;
use App\Enums\LifetimeStatus;
use App\Enums\MapLayout;
use App\Enums\MapSolarsystemStatus;
use App\Enums\MassStatus;
use App\Enums\Permission;
use App\Enums\ShipSize;
use Illuminate\Http\UploadedFile;
use Illuminate\Validation\ValidationException;

final readonly class ParseMapExportFileAction
{
    /**
     * Field specs per section entry. A leading "?" marks the field nullable; the
     * part after ":" parameterises the base type (max length, enum class, list).
     * Validated by hand instead of Laravel wildcard rules because exports carry
     * thousands of entries and rule expansion is quadratic at that size.
     */
    private const array SETTINGS_SPEC = [
        'name' => 'string:255',
        'layout' => 'enum:'.MapLayout::class,
        'allow_layout_override' => 'bool',
        'constant_width_enabled' => 'bool',
        'bookmark_format_wormhole' => 'string:255',
        'bookmark_format_kspace' => 'string:255',
        'bookmark_format_return' => 'string:255',
        'bookmark_alias_scheme' => 'enum:'.AliasScheme::class,
        'bookmark_ignored_alias' => '?string:255',
        'home_solarsystem_id' => '?int',
        'rally_solarsystem_id' => '?int',
    ];

    private const array ACCESS_SPEC = [
        'entity_type' => 'in:character,corporation,alliance',
        'entity_id' => 'int',
        'entity_name' => '?string:255',
        'permission' => 'enum:'.Permission::class,
        'expires_at' => '?date',
    ];

    private const array SOLARSYSTEM_SPEC = [
        'solarsystem_id' => 'int',
        'alias' => '?string:255',
        'position_x' => '?number',
        'position_y' => '?number',
        'pinned' => '?bool',
        'status' => 'enum:'.MapSolarsystemStatus::class,
        'occupier_alias' => '?string:255',
        'notes' => '?string',
    ];

    private const array CONNECTION_SPEC = [
        'from_solarsystem_id' => 'int',
        'to_solarsystem_id' => 'int',
        'wormhole' => '?string:255',
        'type' => 'enum:'.ConnectionType::class,
        'mass_status' => 'enum:'.MassStatus::class,
        'ship_size' => '?enum:'.ShipSize::class,
        'lifetime' => 'enum:'.LifetimeStatus::class,
        'lifetime_updated_at' => '?date',
        'connected_at' => '?date',
        'preserve_mass' => 'bool',
    ];

    private const array SIGNATURE_SPEC = [
        'solarsystem_id' => 'int',
        'signature_id' => '?size:7',
        'category' => '?string:255',
        'type_name' => '?string:255',
        'raw_type_name' => '?string:255',
        'wormhole' => '?string:255',
        'connection_index' => '?int',
        'mass_status' => '?enum:'.MassStatus::class,
        'ship_size' => '?enum:'.ShipSize::class,
        'lifetime' => '?enum:'.LifetimeStatus::class,
        'lifetime_updated_at' => '?date',
    ];

    private const array ROUTE_SPEC = [
        'solarsystem_id' => 'int',
        'is_pinned' => 'bool',
    ];

    private const array IGNORED_SPEC = [
        'solarsystem_id' => 'int',
    ];

    /**
     * Decode and validate an uploaded export file, returning only the requested
     * sections. Throws a ValidationException keyed on `file` for anything wrong
     * with the upload so errors surface in the standard error bag.
     *
     * @param  list<string>  $sections
     * @return array<string, mixed>
     *
     * @throws ValidationException
     */
    public function handle(UploadedFile $file, array $sections, bool $for_new_map = false): array
    {
        $contents = $file->getContent();

        if (! json_validate($contents)) {
            $this->fail('This file is not valid JSON.');
        }

        $data = json_decode($contents, true);

        if (! is_array($data) || ($data['format'] ?? null) !== ExportMapAction::FORMAT) {
            $this->fail('This file is not a wormholesystems map export.');
        }

        if (($data['version'] ?? null) !== ExportMapAction::VERSION) {
            $this->fail('This file was exported by an incompatible version of the application.');
        }

        $available = is_array($data['sections'] ?? null) ? $data['sections'] : [];

        foreach ($sections as $section) {
            if (! array_key_exists($section, $available)) {
                $this->fail(sprintf('The file does not contain the "%s" section.', $section));
            }
        }

        if ($for_new_map && ! in_array('solarsystems', $sections, true)) {
            foreach (['connections', 'signatures'] as $dependent) {
                if (in_array($dependent, $sections, true)) {
                    $this->fail(sprintf('Importing %s into a new map requires the solar systems section.', $dependent));
                }
            }
        }

        if (! is_string($data['map_name'] ?? null) || $data['map_name'] === '' || mb_strlen($data['map_name']) > 255) {
            $this->fail('The file does not contain a valid map name.');
        }

        $data['sections'] = collect($available)
            ->only($sections)
            ->all();

        foreach ($data['sections'] as $section => $value) {
            $this->validateSection((string) $section, $value);
        }

        return $data;
    }

    private function validateSection(string $section, mixed $value): void
    {
        match ($section) {
            'settings' => $this->validateEntry('sections.settings', $value, self::SETTINGS_SPEC),
            'access' => $this->validateList('sections.access', $value, self::ACCESS_SPEC),
            'solarsystems' => $this->validateList('sections.solarsystems', $value, self::SOLARSYSTEM_SPEC),
            'connections' => $this->validateList('sections.connections', $value, self::CONNECTION_SPEC),
            'signatures' => $this->validateList('sections.signatures', $value, self::SIGNATURE_SPEC),
            'routes' => $this->validateRoutes($value),
            default => $this->invalid($section),
        };
    }

    private function validateRoutes(mixed $value): void
    {
        if (! is_array($value)) {
            $this->invalid('sections.routes');
        }

        $this->validateList('sections.routes.route_solarsystems', $value['route_solarsystems'] ?? null, self::ROUTE_SPEC);
        $this->validateList('sections.routes.ignored_solarsystems', $value['ignored_solarsystems'] ?? null, self::IGNORED_SPEC);
    }

    /**
     * @param  array<string, string>  $spec
     */
    private function validateList(string $path, mixed $entries, array $spec): void
    {
        if (! is_array($entries) || ! array_is_list($entries)) {
            $this->invalid($path);
        }

        foreach ($entries as $index => $entry) {
            $this->validateEntry(sprintf('%s.%d', $path, $index), $entry, $spec);
        }
    }

    /**
     * @param  array<string, string>  $spec
     */
    private function validateEntry(string $path, mixed $entry, array $spec): void
    {
        if (! is_array($entry)) {
            $this->invalid($path);
        }

        foreach ($spec as $field => $type) {
            $nullable = str_starts_with($type, '?');
            $value = $entry[$field] ?? null;

            if ($value === null) {
                if (! $nullable) {
                    $this->invalid(sprintf('%s.%s', $path, $field));
                }

                continue;
            }

            if (! $this->matchesType($value, mb_ltrim($type, '?'))) {
                $this->invalid(sprintf('%s.%s', $path, $field));
            }
        }
    }

    private function matchesType(mixed $value, string $type): bool
    {
        [$base, $param] = array_pad(explode(':', $type, 2), 2, null);

        return match ($base) {
            'int' => is_int($value),
            'number' => is_int($value) || is_float($value),
            'bool' => is_bool($value) || $value === 0 || $value === 1,
            'date' => is_string($value) && strtotime($value) !== false,
            'string' => is_string($value) && ($param === null || mb_strlen($value) <= (int) $param),
            'size' => is_string($value) && mb_strlen($value) === (int) $param,
            'in' => is_string($value) && in_array($value, explode(',', (string) $param), true),
            'enum' => is_string($value) && $param::tryFrom($value) !== null,
            default => false,
        };
    }

    private function invalid(string $path): never
    {
        $this->fail(sprintf('The file contains invalid data at %s.', $path));
    }

    private function fail(string $message): never
    {
        throw ValidationException::withMessages(['file' => $message]);
    }
}
