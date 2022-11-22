<script lang="ts">
	import { inview } from 'svelte-inview';
	import { fly } from 'svelte/transition';
	import type { Options } from 'svelte-inview';

	export let id = 'id';
	let isInView = false;
	const options: Options = {
		rootMargin: '-50%'
	};
</script>

<div
	{id}
	class="w-screen h-screen"
	use:inview={options}
	on:change={(e) => {
		isInView = e.detail.inView;
	}}
>
	{#if isInView}
		<div
			class="flex justify-center items-center w-full h-full p-10"
			transition:fly={{ y: 100, opacity: 0, delay: 0 }}
		>
			<ul class="w-full h-auto full md:hidden flex flex-col items-center gap-4">
				<li class="flex flex-col gap-1 text-white snap-center max-h-[90vh]">
					<span>ERPWeb</span>
					<img
						class="rounded-xl"
						alt="Pantalla de inicio de un sistema ERP"
						src="/images/erpweb-v.PNG"
					/>
				</li>
			</ul>
			<ul
				class="w-full h-auto max-w-[70vw] hidden md:flex items-center gap-16 snap-mandatory snap-x overflow-x-scroll p-2 text-2xl"
			>
				<li
					class="flex flex-col gap-1 text-white snap-center max-w-[70vw] shrink-0 overflow-hidden"
				>
					<span>ERPWeb</span>
					<img
						class="rounded-3xl"
						alt="Pantalla de inicio de un sistema ERP"
						src="/images/erpweb.PNG"
					/>
				</li>
				<li
					class="flex flex-col gap-1 text-white snap-center max-w-[70vw] shrink-0 overflow-hidden"
				>
					<span>ERPWeb</span>
					<img
						class="rounded-3xl"
						alt="Pantalla de inicio de un sistema ERP"
						src="/images/erpweb.PNG"
					/>
				</li>
				<li
					class="flex flex-col gap-1 text-white snap-center max-w-[70vw] shrink-0 overflow-hidden"
				>
					<span>ERPWeb</span>
					<img
						class="rounded-3xl"
						alt="Pantalla de inicio de un sistema ERP"
						src="/images/erpweb.PNG"
					/>
				</li>
			</ul>
		</div>
	{/if}
</div>
