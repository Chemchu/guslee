<script lang="ts">
	import { fly } from 'svelte/transition';

	export let title = 'Titulo!';
	export let msg =
		'Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.';

	let mouseIn = false;
</script>

<div class="flex w-full h-full gap-4" on:mouseleave={() => (mouseIn = false)}>
	<div
		class="flex flex-col justify-start gap-6 border border-white w-1/2 h-full bg-black rounded-md px-4 py-5"
		on:mouseenter={() => (mouseIn = true)}
	>
		<h1 class="text-3xl">{title}</h1>
		<p class="text-lg">{msg}</p>
	</div>
	{#if mouseIn}
		<div
			class="flex flex-col grow h-full bg-black border rounded-md px-4 py-5"
			transition:fly={{ x: 100, opacity: 0 }}
		>
			MouseIn!
		</div>
	{/if}
</div>
