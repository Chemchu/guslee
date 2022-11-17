<script lang="ts">
	import { fade } from 'svelte/transition';
	import { onMount } from 'svelte';
	import SplashScreen from '$lib/components/SplashScreen.svelte';
	import AboutMe from '$lib/components/AboutMe.svelte';
	import Projects from '$lib/components/Projects.svelte';
	import ContactMe from '$lib/components/ContactMe.svelte';

	let mounted = false;
	onMount(async () => {
		const p = new Promise(() => {
			setTimeout(() => {
				mounted = true;
			}, 2000);
		});

		await p;
	});

	let showBackFace = false;
	const handleClick = (e: any) => {
		showBackFace = !showBackFace;
		console.log(e);
	};
</script>

{#if !mounted}
	<SplashScreen />
{:else}
	<div class="fixed w-full h-screen bg-black -z-20">
		<video
			class="w-full h-full object-cover -z-10 bg-black"
			src="/videos/background_video.mp4"
			muted
			loop
			autoplay
			transition:fade={{ delay: 500, duration: 1000 }}
		>
			<track kind="captions" />
		</video>
	</div>
	<div class="flex flex-col w-full h-screen items-center justify-center text-white">
		<div class="flex flex-col items-center justify-center w-10/12 h-5/6 py-10 md:p-16">
			<div
				class="flex flex-col items-center justify-between w-full h-full"
				transition:fade={{ delay: 1700, duration: 1500 }}
			>
				<span class="s-title w-full text-start">Me llamo</span>
				<div class="container">
					<div class="card" on:click={handleClick}>
						<span class="front-face b-title w-full text-center font-extralight">Gustavo</span>
						<span class="back-face flex justify-between items-center">
							<div class="bg-red-500">
								<img src="/images/linkedin.svg" alt="LinkedIn logo" />
							</div>
							<div>GitHub</div>
							<div>CV</div>
						</span>
					</div>
				</div>
				<span class="m-title w-full text-end font-bold">Lee</span>
			</div>
		</div>
	</div>
	<div class="overflow-x-hidden">
		<AboutMe />
		<Projects />
		<ContactMe />
	</div>
{/if}

<style>
	.container {
		width: 100%;
		height: 100%;
		position: relative;
	}

	.card {
		position: absolute;
		width: 100%;
		height: 100%;
		transform-style: preserve-3d;
		transition: all 0.5s ease;
	}

	.front-face,
	.back-face {
		position: absolute;
		width: 100%;
		height: 100%;
		backface-visibility: hidden;
	}

	.back-face {
		transform: rotateY(180deg);
	}

	.card:hover {
		transform: rotateX(180deg);
	}

	.b-title {
		font-size: 18vw;
		line-height: 100%;
	}

	.m-title {
		font-size: 10vw;
		line-height: 100%;
	}

	.s-title {
		font-size: 4vw;
		line-height: 100%;
	}
</style>
