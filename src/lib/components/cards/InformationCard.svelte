<script lang="ts">
	import { inview } from 'svelte-inview';

	import { fly } from 'svelte/transition';

	export let title = 'Titulo!';
	export let expandedTitle = 'Titulo 2!';
	export let msg =
		'Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.';
	export let expandedMsg =
		'Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.';

	let mouseIn = false;
	let inView = false;
</script>

<div
	class="flex w-full h-full gap-4"
	use:inview
	on:change={(e) => (inView = e.detail.inView)}
	on:mouseleave={() => (mouseIn = false)}
>
	{#if inView}
		<div
			class="flex flex-col justify-start gap-6 border border-white shrink-0 w-1/2 h-full bg-black rounded-2xl px-4 py-5"
			on:mouseenter={() => (mouseIn = true)}
			transition:fly={{ x: -100, opacity: 0, delay: 300 }}
		>
			<h1 class="text-3xl">{title}</h1>
			<p class="text-lg">{msg}</p>
		</div>
		{#if mouseIn}
			<div
				class="flex flex-col w-1/2 h-full bg-black border rounded-2xl px-4 py-5"
				transition:fly={{ x: 100, opacity: 0 }}
			>
				<span>{expandedTitle}</span>
				<span>{expandedMsg}</span>
			</div>
		{/if}
	{/if}
</div>
