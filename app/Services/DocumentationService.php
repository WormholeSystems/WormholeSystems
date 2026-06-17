<?php

declare(strict_types=1);

namespace App\Services;

use Illuminate\Support\Arr;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Str;
use Symfony\Component\Yaml\Yaml;

final class DocumentationService
{
    /**
     * @var Collection<int, array{slug: string, title: string, category: string, body: string, url: string, path: string}>|null
     */
    private ?Collection $cachedEntries = null;

    /**
     * The grouped navigation tree used to render the sidebar.
     *
     * @return list<array{title: string, pages: list<array{title: string, slug: string, url: string}>}>
     */
    public function navigation(): array
    {
        return $this->entries()
            ->groupBy('category')
            ->map(fn (Collection $pages, string $category): array => [
                'title' => $category,
                'pages' => $pages
                    ->map(fn (array $page): array => Arr::only($page, ['title', 'slug', 'url']))
                    ->values()
                    ->all(),
            ])
            ->values()
            ->all();
    }

    /**
     * Resolve a single documentation page by slug, falling back to the first page.
     *
     * @return array{title: string, category: string, slug: string, html: string, markdown: string, edit_url: string, prev: ?array{title: string, slug: string, url: string}, next: ?array{title: string, slug: string, url: string}}
     */
    public function page(?string $slug): array
    {
        $entries = $this->entries();

        abort_if($entries->isEmpty(), 404);

        $index = $slug === null
            ? 0
            : $entries->search(fn (array $entry): bool => $entry['slug'] === $slug);

        abort_if($index === false, 404);

        $entry = $entries[$index];

        return [
            'title' => $entry['title'],
            'category' => $entry['category'],
            'slug' => $entry['slug'],
            'html' => $this->renderHtml($entry['body']),
            'markdown' => mb_trim($entry['body']),
            'edit_url' => mb_rtrim((string) config('documentation.edit_base_url'), '/').'/'.$entry['path'],
            'prev' => $this->link($entries->get($index - 1)),
            'next' => $this->link($entries->get($index + 1)),
        ];
    }

    /**
     * Read and order every documentation entry from disk.
     *
     * @return Collection<int, array{slug: string, title: string, category: string, body: string, url: string, path: string}>
     */
    private function entries(): Collection
    {
        if ($this->cachedEntries instanceof Collection) {
            return $this->cachedEntries;
        }

        $base = resource_path('docs');

        if (! File::isDirectory($base)) {
            return $this->cachedEntries = collect();
        }

        return $this->cachedEntries = collect(File::directories($base))
            ->sort()
            ->flatMap(function (string $directory): Collection {
                $categoryDir = basename($directory);
                $categorySlug = $this->stripOrderPrefix($categoryDir);
                $categoryTitle = $this->humanize($categorySlug);

                return collect(File::files($directory))
                    ->filter(fn ($file): bool => $file->getExtension() === 'md')
                    ->sortBy(fn ($file): string => $file->getFilename())
                    ->map(function ($file) use ($categoryDir, $categorySlug, $categoryTitle): array {
                        [$frontmatter, $body] = $this->parse($file->getContents());

                        $pageSlug = $this->stripOrderPrefix(pathinfo($file->getFilename(), PATHINFO_FILENAME));
                        $slug = "{$categorySlug}/{$pageSlug}";

                        return [
                            'slug' => $slug,
                            'title' => $frontmatter['title'] ?? $this->firstHeading($body) ?? $this->humanize($pageSlug),
                            'category' => $frontmatter['category'] ?? $categoryTitle,
                            'body' => $body,
                            'url' => '/documentation/'.$slug,
                            'path' => 'resources/docs/'.$categoryDir.'/'.$file->getFilename(),
                        ];
                    })
                    ->values();
            })
            ->values();
    }

    /**
     * Render Markdown to HTML, opening external links in a new tab.
     */
    private function renderHtml(string $body): string
    {
        $html = Str::markdown($body, ['html_input' => 'strip', 'allow_unsafe_links' => false]);

        return (string) preg_replace(
            '/<a href="(https?:\/\/)/',
            '<a target="_blank" rel="noopener noreferrer" href="$1',
            $html
        );
    }

    /**
     * @return array{0: array<string, mixed>, 1: string}
     */
    private function parse(string $raw): array
    {
        if (preg_match('/^---\s*\n(.*?)\n---\s*\n(.*)$/s', $raw, $matches) === 1) {
            $frontmatter = Yaml::parse($matches[1]);

            return [is_array($frontmatter) ? $frontmatter : [], $matches[2]];
        }

        return [[], $raw];
    }

    private function firstHeading(string $body): ?string
    {
        if (preg_match('/^#\s+(.+)$/m', $body, $matches) === 1) {
            return mb_trim($matches[1]);
        }

        return null;
    }

    private function stripOrderPrefix(string $name): string
    {
        return (string) preg_replace('/^\d+[-_]/', '', $name);
    }

    private function humanize(string $slug): string
    {
        return Str::of($slug)->replace('-', ' ')->ucfirst()->value();
    }

    /**
     * @param  array{slug: string, title: string, category: string, body: string, url: string}|null  $entry
     * @return array{title: string, slug: string, url: string}|null
     */
    private function link(?array $entry): ?array
    {
        return $entry === null ? null : Arr::only($entry, ['title', 'slug', 'url']);
    }
}
