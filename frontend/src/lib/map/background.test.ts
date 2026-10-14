import { describe, expect, it } from 'vitest';

import { canvasBackground } from './background';

describe('canvasBackground', () => {
	const image = '/api/maps/4/background-image?v=9-1.png';
	const picture = `background-image: url("${image}"); background-size: cover; background-repeat: no-repeat; background-position: center center;`;

	it('paints a grid-mode picture onto the world, behind the grid lines', () => {
		const { world, viewport } = canvasBackground({
			gridLines: true,
			cell: 20,
			image,
			mode: 'grid',
		});
		expect(world).toContain(`url("${image}")`);
		expect(world).toContain('background-size: 20px 20px, 20px 20px, cover;');
		expect(world).toContain('background-repeat: repeat, repeat, no-repeat;');
		expect(world.indexOf('linear-gradient')).toBeLessThan(world.indexOf('url('));
		expect(viewport).toBe('');
	});

	it('pins a viewport-mode picture to the panel and leaves the world with only its lines', () => {
		const { world, viewport } = canvasBackground({
			gridLines: true,
			cell: 20,
			image,
			mode: 'viewport',
		});
		expect(world).not.toContain('url(');
		expect(world).toContain('linear-gradient');
		expect(viewport).toBe(picture);
	});

	it('draws nothing at all without lines or a picture', () => {
		expect(canvasBackground({ gridLines: false, cell: 20, image: null, mode: 'grid' })).toEqual({
			world: '',
			viewport: '',
		});
	});

	it('keeps a locked layout free of grid lines but still shows the picture', () => {
		const { world } = canvasBackground({ gridLines: false, cell: 20, image, mode: 'grid' });
		expect(world).toBe(picture);
	});
});
