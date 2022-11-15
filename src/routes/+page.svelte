<script lang="ts">
	import { fade } from 'svelte/transition';
	import { onMount } from 'svelte';
	import SplashScreen from '$lib/components/SplashScreen.svelte';

	let mounted = false;
	onMount(async () => {
		const p = new Promise(() => {
			setTimeout(() => {
				mounted = true;
			}, 2000);
		});

		await p;
	});
</script>

{#if !mounted}
	<SplashScreen />
{:else}
	<div class="absolute w-full h-screen bg-black -z-20">
		<video
			class="w-full h-full object-cover -z-10 bg-black py-20"
			src="/videos/background_video5.mp4"
			muted
			loop
			autoplay
			transition:fade={{ delay: 500, duration: 1000 }}
		>
			<track kind="captions" />
		</video>
	</div>
	<div class="flex flex-col w-full h-screen items-center justify-center">
		<div class="text-white w-10/12 py-10 md:p-16">
			<div class="flex flex-col w-full" transition:fade={{ delay: 1700, duration: 1500 }}>
				<span class="s-title w-full text-start">Me llamo</span>
				<span class="b-title w-full text-center font-extralight">Gustavo</span>
				<span class="m-title w-full text-end font-bold">Lee</span>
			</div>
		</div>
	</div>
	<div class="bg-black w-full h-screen">
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
		<div class="relative text-white">Coño</div>
	</div>
{/if}

<style>
	.b-title {
		font-size: 16vw;
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
