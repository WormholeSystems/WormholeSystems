<!DOCTYPE html>
<html lang="{{ str_replace('_', '-', app()->getLocale()) }}" @class(['dark' => ($appearance ?? 'system') == 'dark'])>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">

    {{-- Inline script to detect system dark mode preference and apply it immediately --}}
    <script>
        (function() {
            const appearance = '{{ $appearance ?? "system" }}';

            if (appearance === 'system') {
                const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

                if (prefersDark) {
                    document.documentElement.classList.add('dark');
                }
            }
        })();
    </script>

    {{-- Inline style to set the HTML background color based on our theme in app.css --}}
    <style>
        html {
            background-color: oklch(1 0 0);
        }

        html.dark {
            background-color: oklch(0.145 0 0);
        }
    </style>

    <title inertia>{{ config('app.name', 'Laravel') }}</title>

    <!-- PWA Meta Tags -->
    <meta name="application-name" content="wormhole.systems">
    <meta name="description"
          content="Advanced wormhole mapping and tracking system for EVE Online. Navigate dangerous wormhole space with real-time intel, signature tracking, and collaborative mapping tools.">
    <meta name="keywords"
          content="EVE Online, wormhole, mapping, tracking, signatures, intel, navigation, space, gaming">
    <meta name="apple-mobile-web-app-capable" content="yes">
    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
    <meta name="apple-mobile-web-app-title" content="wormhole.systems">
    <meta name="mobile-web-app-capable" content="yes">
    <meta name="msapplication-TileColor" content="#000000">
    <meta name="msapplication-tap-highlight" content="no">
    <meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)">
    <meta name="theme-color" content="#000000" media="(prefers-color-scheme: dark)">

        <!-- Open Graph / Social Media tags are handled by SeoHead.vue component -->

    <!-- Icons -->
    <!-- Light mode favicons -->
    <link rel="icon" type="image/png" sizes="32x32" href="/favicon.png" media="(prefers-color-scheme: light)">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png" media="(prefers-color-scheme: light)">

    <!-- Dark mode favicons -->
    <link rel="icon" type="image/png" sizes="32x32" href="/favicon-dark.png" media="(prefers-color-scheme: dark)">
    <link rel="apple-touch-icon" href="/apple-touch-icon-dark.png" media="(prefers-color-scheme: dark)">

    <!-- Default favicon (no preference) -->
    <link rel="icon" type="image/png" href="/favicon.png" media="(prefers-color-scheme: no-preference)">

    <!-- PWA Manifest -->
    <link rel="manifest" href="/build/manifest.webmanifest">

    @vite(['resources/js/app.ts', "resources/js/pages/{$page['component']}.vue"])
    @inertiaHead
</head>
<body class="font-sans antialiased">
@inertia
</body>
</html>
