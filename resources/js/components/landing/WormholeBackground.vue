<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';

interface Particle {
    angle: number;
    radius: number;
    speed: number;
    size: number;
    hue: number;
    trail: number;
}

const canvas = ref<HTMLCanvasElement | null>(null);
const ready = ref(false);

let ctx: CanvasRenderingContext2D | null = null;
let frame = 0;
let particles: Particle[] = [];
let stars: { x: number; y: number; r: number; twinkle: number }[] = [];
let width = 0;
let height = 0;
let dpr = 1;
let isDark = true;
let running = false;
let prefersReduced = false;

let observer: MutationObserver | null = null;
let resizeObserver: ResizeObserver | null = null;

const PALETTE = [22, 32, 42]; // one warm family: the accretion disk's oranges

function syncTheme(): void {
    isDark = typeof document !== 'undefined' && document.documentElement.classList.contains('dark');
}

function center(): { cx: number; cy: number } {
    // Offset the vortex toward the upper-right for an asymmetric composition.
    return { cx: width * 0.72, cy: height * 0.34 };
}

function maxRadius(): number {
    return Math.hypot(width, height) * 0.9;
}

function spawn(seed = false): Particle {
    const r = seed ? Math.random() * maxRadius() : maxRadius() * (0.85 + Math.random() * 0.15);
    return {
        angle: Math.random() * Math.PI * 2,
        radius: r,
        speed: 0.0004 + Math.random() * 0.0008,
        size: 0.6 + Math.random() * 1.8,
        hue: PALETTE[Math.floor(Math.random() * PALETTE.length)],
        trail: 6 + Math.random() * 22,
    };
}

function build(): void {
    const area = width * height;
    const count = Math.min(260, Math.max(100, Math.round(area / 8000)));
    particles = Array.from({ length: count }, () => spawn(true));

    const starCount = Math.min(300, Math.max(90, Math.round(area / 8000)));
    stars = Array.from({ length: starCount }, () => ({
        x: Math.random() * width,
        y: Math.random() * height,
        r: Math.random() * 1.6 + 0.4,
        twinkle: Math.random() * Math.PI * 2,
    }));
}

function resize(): void {
    if (!canvas.value) {
        return;
    }
    dpr = Math.min(window.devicePixelRatio || 1, 2);
    width = canvas.value.clientWidth;
    height = canvas.value.clientHeight;
    canvas.value.width = Math.floor(width * dpr);
    canvas.value.height = Math.floor(height * dpr);
    ctx?.setTransform(dpr, 0, 0, dpr, 0, 0);
    build();
}

function drawCore(): void {
    if (!ctx) {
        return;
    }
    const { cx, cy } = center();
    const pulse = 1 + Math.sin(frame * 0.01) * 0.06;
    const coreR = Math.min(width, height) * 0.22 * pulse;

    const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, coreR);
    if (isDark) {
        glow.addColorStop(0, 'rgba(255, 214, 165, 0.55)');
        glow.addColorStop(0.25, 'rgba(251, 146, 60, 0.26)');
        glow.addColorStop(0.6, 'rgba(194, 65, 12, 0.12)');
        glow.addColorStop(1, 'rgba(0, 0, 0, 0)');
    } else {
        glow.addColorStop(0, 'rgba(234, 88, 12, 0.22)');
        glow.addColorStop(0.4, 'rgba(251, 146, 60, 0.10)');
        glow.addColorStop(1, 'rgba(255, 255, 255, 0)');
    }
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(cx, cy, coreR, 0, Math.PI * 2);
    ctx.fill();

    // The event horizon itself: a dark disc swallowing the glow's centre.
    ctx.fillStyle = isDark ? 'rgba(5, 5, 8, 0.9)' : 'rgba(250, 250, 250, 0.9)';
    ctx.beginPath();
    ctx.arc(cx, cy, coreR * 0.3, 0, Math.PI * 2);
    ctx.fill();
}

function render(): void {
    if (!ctx) {
        return;
    }
    ctx.clearRect(0, 0, width, height);

    const { cx, cy } = center();

    // Distant twinkling stars.
    for (const s of stars) {
        const t = 0.4 + 0.6 * Math.abs(Math.sin(frame * 0.01 + s.twinkle));
        ctx.fillStyle = isDark ? `rgba(226, 232, 240, ${0.12 + t * 0.28})` : `rgba(15, 23, 42, ${0.05 + t * 0.1})`;
        ctx.beginPath();
        ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
        ctx.fill();
    }

    drawCore();

    // Spiralling accretion particles.
    const inner = Math.min(width, height);
    const fade_start = inner * 0.22;
    const kill_radius = inner * 0.06;

    for (const p of particles) {
        p.angle += p.speed * (1 + ((maxRadius() - p.radius) / maxRadius()) * 5);
        p.radius -= p.speed * p.radius * 0.5 + 0.06;

        if (p.radius < kill_radius) {
            Object.assign(p, spawn());
        }

        const x = cx + Math.cos(p.angle) * p.radius;
        const y = cy + Math.sin(p.angle) * p.radius * 0.62; // squash for perspective
        const tx = cx + Math.cos(p.angle - 0.18) * (p.radius + p.trail);
        const ty = cy + Math.sin(p.angle - 0.18) * (p.radius + p.trail) * 0.62;

        const depth = 1 - p.radius / maxRadius();
        const core_fade = Math.min(1, Math.max(0, (p.radius - kill_radius) / (fade_start - kill_radius)));
        const alpha = (isDark ? 0.6 : 0.35) * (0.25 + depth) * core_fade;
        const light = isDark ? 65 : 45;

        ctx.strokeStyle = `hsla(${p.hue}, 90%, ${light}%, ${alpha})`;
        ctx.lineWidth = p.size * (0.4 + depth);
        ctx.beginPath();
        ctx.moveTo(tx, ty);
        ctx.lineTo(x, y);
        ctx.stroke();
    }
}

function loop(): void {
    if (!running) {
        return;
    }
    frame += 1;
    render();
    requestAnimationFrame(loop);
}

function start(): void {
    if (running || prefersReduced) {
        return;
    }
    running = true;
    requestAnimationFrame(loop);
}

function stop(): void {
    running = false;
}

function onVisibility(): void {
    if (document.hidden) {
        stop();
    } else {
        start();
    }
}

onMounted(() => {
    if (!canvas.value) {
        return;
    }
    ctx = canvas.value.getContext('2d');
    syncTheme();
    prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    resize();

    if (prefersReduced) {
        // Honour the user's preference with a single static frame.
        render();
    } else {
        start();
    }

    // The first frame lands a beat after mount; ease the field in instead of popping.
    requestAnimationFrame(() => {
        ready.value = true;
    });

    observer = new MutationObserver(() => {
        syncTheme();
        if (prefersReduced) {
            render();
        }
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });

    resizeObserver = new ResizeObserver(() => {
        resize();
        if (prefersReduced) {
            render();
        }
    });
    resizeObserver.observe(canvas.value);

    document.addEventListener('visibilitychange', onVisibility);
});

onBeforeUnmount(() => {
    stop();
    observer?.disconnect();
    resizeObserver?.disconnect();
    document.removeEventListener('visibilitychange', onVisibility);
});
</script>

<template>
    <div class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
        <div class="bg-void absolute inset-0" />
        <div class="aurora absolute inset-0" />
        <div class="canvas-blur absolute inset-0" :class="{ 'canvas-blur--ready': ready }">
            <canvas ref="canvas" class="absolute inset-0 h-full w-full" />
        </div>
        <div class="bg-vignette absolute inset-0" />
    </div>
</template>

<style scoped>
/* Blur lives on a wrapper: Chromium can drop a filter set directly on a
   canvas once it promotes the repainting canvas to its own compositor layer.
   The field fades in once the first frame has rendered. */
.canvas-blur {
    filter: blur(10px);
    transform: translateZ(0);
    opacity: 0;
    transition: opacity 2.2s ease-out;
}

.canvas-blur--ready {
    opacity: 1;
}

.bg-void {
    background:
        radial-gradient(120% 90% at 72% 30%, rgba(251, 146, 60, 0.1), transparent 55%),
        radial-gradient(90% 70% at 15% 90%, rgba(194, 65, 12, 0.07), transparent 50%), var(--background);
}

.bg-vignette {
    background: radial-gradient(120% 120% at 50% 45%, transparent 42%, color-mix(in oklab, var(--background) 80%, transparent) 100%);
}

/* Slow-drifting ambient colour wash behind the particle field. */
.aurora {
    background:
        radial-gradient(38% 38% at 28% 32%, color-mix(in oklab, var(--color-orange-500) 16%, transparent), transparent 70%),
        radial-gradient(34% 34% at 76% 66%, color-mix(in oklab, var(--color-red-600) 11%, transparent), transparent 70%);
    filter: blur(44px);
    animation: aurora-drift 26s ease-in-out infinite alternate;
    will-change: transform;
}

@keyframes aurora-drift {
    from {
        transform: translate3d(-3%, -2%, 0) scale(1);
    }
    to {
        transform: translate3d(4%, 3%, 0) scale(1.12);
    }
}

@media (prefers-reduced-motion: reduce) {
    .aurora {
        animation: none;
    }
}
</style>
