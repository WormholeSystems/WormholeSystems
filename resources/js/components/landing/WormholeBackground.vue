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

const PALETTE = [190, 200, 280, 28]; // cyan, blue, violet, amber — wormhole class hues

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
        speed: 0.0009 + Math.random() * 0.0016,
        size: 0.6 + Math.random() * 1.8,
        hue: PALETTE[Math.floor(Math.random() * PALETTE.length)],
        trail: 6 + Math.random() * 22,
    };
}

function build(): void {
    const area = width * height;
    const count = Math.min(420, Math.max(140, Math.round(area / 5200)));
    particles = Array.from({ length: count }, () => spawn(true));

    const starCount = Math.min(260, Math.max(80, Math.round(area / 9000)));
    stars = Array.from({ length: starCount }, () => ({
        x: Math.random() * width,
        y: Math.random() * height,
        r: Math.random() * 1.1 + 0.2,
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
    const pulse = 1 + Math.sin(frame * 0.02) * 0.06;
    const coreR = Math.min(width, height) * 0.22 * pulse;

    const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, coreR);
    if (isDark) {
        glow.addColorStop(0, 'rgba(125, 240, 255, 0.50)');
        glow.addColorStop(0.25, 'rgba(56, 189, 248, 0.22)');
        glow.addColorStop(0.6, 'rgba(99, 102, 241, 0.10)');
        glow.addColorStop(1, 'rgba(0, 0, 0, 0)');
    } else {
        glow.addColorStop(0, 'rgba(8, 145, 178, 0.20)');
        glow.addColorStop(0.4, 'rgba(56, 189, 248, 0.10)');
        glow.addColorStop(1, 'rgba(255, 255, 255, 0)');
    }
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(cx, cy, coreR, 0, Math.PI * 2);
    ctx.fill();

    // Event-horizon ring.
    ctx.strokeStyle = isDark ? 'rgba(165, 243, 252, 0.35)' : 'rgba(14, 116, 144, 0.30)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(cx, cy, coreR * 0.42, 0, Math.PI * 2);
    ctx.stroke();
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
        ctx.fillStyle = isDark ? `rgba(226, 232, 240, ${0.06 + t * 0.18})` : `rgba(15, 23, 42, ${0.03 + t * 0.07})`;
        ctx.beginPath();
        ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
        ctx.fill();
    }

    drawCore();

    // Spiralling accretion particles.
    for (const p of particles) {
        p.angle += p.speed * (1 + ((maxRadius() - p.radius) / maxRadius()) * 5);
        p.radius -= p.speed * p.radius * 0.9 + 0.15;

        if (p.radius < Math.min(width, height) * 0.05) {
            Object.assign(p, spawn());
        }

        const x = cx + Math.cos(p.angle) * p.radius;
        const y = cy + Math.sin(p.angle) * p.radius * 0.62; // squash for perspective
        const tx = cx + Math.cos(p.angle - 0.18) * (p.radius + p.trail);
        const ty = cy + Math.sin(p.angle - 0.18) * (p.radius + p.trail) * 0.62;

        const depth = 1 - p.radius / maxRadius();
        const alpha = (isDark ? 0.55 : 0.32) * (0.25 + depth);
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
        <canvas ref="canvas" class="absolute inset-0 h-full w-full" />
        <div class="bg-vignette absolute inset-0" />
    </div>
</template>

<style scoped>
.bg-void {
    background:
        radial-gradient(120% 90% at 72% 30%, rgba(14, 165, 233, 0.12), transparent 55%),
        radial-gradient(90% 70% at 15% 90%, rgba(124, 58, 237, 0.1), transparent 50%), var(--background);
}

.bg-vignette {
    background: radial-gradient(120% 120% at 50% 45%, transparent 35%, color-mix(in oklab, var(--background) 78%, transparent) 100%);
}

/* Slow-drifting ambient colour wash behind the particle field. */
.aurora {
    background:
        radial-gradient(38% 38% at 28% 32%, color-mix(in oklab, var(--color-cyan-500) 16%, transparent), transparent 70%),
        radial-gradient(34% 34% at 76% 66%, color-mix(in oklab, var(--color-violet-500) 14%, transparent), transparent 70%);
    filter: blur(40px);
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
