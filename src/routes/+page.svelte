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

	let toggleClick = false;
	let outEnded = true;
	const handleClick = () => {
		toggleClick = !toggleClick;
		outEnded = false;
	};

	const handleOutEnded = () => {
		outEnded = true;
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
				<div class="card cursor-pointer" on:click={handleClick} on:keydown={handleClick}>
					{#if !toggleClick && outEnded}
						<span
							class="text-color b-title w-full text-center font-extralight"
							transition:fade
							on:outroend={handleOutEnded}
						>
							Gustavo
						</span>
					{/if}
					{#if toggleClick && outEnded}
						<span transition:fade on:outroend={handleOutEnded}>Hola</span>
					{/if}
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
