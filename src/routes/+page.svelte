<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import init, { WgpuContext,CanvasProxy } from '../../rust_renderer/pkg/rust_renderer';
	import * as m from '$lib/paraglide/messages';
	import { setLocale,getLocale } from '$lib/paraglide/runtime';

	let canvas: HTMLCanvasElement;
	let container: HTMLDivElement;
	let context: WgpuContext | null = null;
	let observer: ResizeObserver;

	onMount(async () => {
		await init();
		const canvasOrigin = document.createElement('canvas');
		const canvasProxy = new CanvasProxy(canvasOrigin);
		context = await new WgpuContext(canvasProxy, '/placeholder.png');

		function render() {
			if (context) {
				context.render();
			}
			requestAnimationFrame(render);
		}
		render();

		observer = new ResizeObserver((entries) => {
			for (const entry of entries) {
				const { width, height } = entry.contentRect;
				if (context) {
					canvas.width = width;
					canvas.height = height;
					
				}
			}
		});

		observer.observe(container);
	});

	onDestroy(() => {
		if (observer) {
			observer.disconnect();
		}
	});
</script>

<svelte:head>
	<title>{m.title()}</title>
</svelte:head>

<div bind:this={container} class="container">
	<canvas bind:this={canvas} class="background-canvas"></canvas>
	<div class="card">
		<h1>{m.greeting()}</h1>
		<p>{m.introduction()}</p>

		<div class="language-switcher">
			<p>{m.change_language()}</p>
			<button on:click={() => setLocale('ko')} class:active={getLocale() === 'ko'}>한국어</button>
			<button on:click={() => setLocale('en')} class:active={getLocale() === 'en'}>English</button>
			<button on:click={() => setLocale('ja')} class:active={getLocale() === 'ja'}>日本語</button>
		</div>
	</div>
</div>

<style>
	.container {
		position: relative;
		width: 100vw;
		height: 100vh;
		display: flex;
		justify-content: center;
		align-items: center;
		overflow: hidden;
	}

	.background-canvas {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		z-index: -1; /* Place canvas behind the card */
	}

	.card {
		background-color: rgba(20, 20, 30, 0.8);
		color: white;
		padding: 2rem 3rem;
		border-radius: 12px;
		border: 1px solid rgba(255, 255, 255, 0.1);
		backdrop-filter: blur(10px);
		max-width: 90%;
		width: 600px;
		text-align: center;
	}

	h1 {
		font-size: 2.5rem;
		margin-bottom: 1rem;
	}

	.language-switcher {
		margin-top: 2rem;
		padding-top: 1rem;
		border-top: 1px solid rgba(255, 255, 255, 0.2);
	}

	.language-switcher button {
		margin: 0 0.5rem;
		padding: 0.5rem 1rem;
		border: 1px solid #ccc;
		border-radius: 4px;
		cursor: pointer;
		background-color: #f0f0f0;
		color: #333;
		transition: all 0.2s ease;
	}

	.language-switcher button.active {
		background-color: #007bff;
		color: white;
		border-color: #007bff;
	}

	.language-switcher button:hover {
		transform: translateY(-2px);
	}
</style>
