<script lang="ts">
	// Counts up to a number once it is scrolled into view. The final value is what renders
	// without JS and what stays on screen afterwards, so the animation is decoration over a
	// correct number rather than the only way to get one.
	let { value }: { value: number } = $props();

	const DURATION = 900;

	let el = $state<HTMLElement | null>(null);
	// Null until the animation takes over, so the server and the first paint both show the
	// real figure and only a running animation ever shows anything else.
	let animated = $state<number | null>(null);
	const shown = $derived(animated ?? value);

	$effect(() => {
		if (!el) return;
		if (
			typeof IntersectionObserver === 'undefined' ||
			window.matchMedia('(prefers-reduced-motion: reduce)').matches
		) {
			return;
		}

		let frame = 0;
		const io = new IntersectionObserver(
			(entries) => {
				if (!entries.some((e) => e.isIntersecting)) return;
				io.disconnect();
				const start = performance.now();
				const step = (now: number) => {
					const t = Math.min(1, (now - start) / DURATION);
					// Fast out of the gate and easing into the real figure.
					animated = t < 1 ? Math.round(value * (1 - Math.pow(1 - t, 3))) : null;
					if (t < 1) frame = requestAnimationFrame(step);
				};
				animated = 0;
				frame = requestAnimationFrame(step);
			},
			{ threshold: 0.4 },
		);
		io.observe(el);
		return () => {
			io.disconnect();
			cancelAnimationFrame(frame);
		};
	});
</script>

<span bind:this={el} class="tabular-nums">{shown.toLocaleString('en-US')}</span>
