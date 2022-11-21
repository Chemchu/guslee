<script>
	import InformationCard from './cards/InformationCard.svelte';
	import { inview } from 'svelte-inview';

	const title = 'Fullstack developer';
	const msg = 'Graduado en ingeniería informática por la Universitat Politècnica de València';
</script>

<div class="w-screen h-screen bg-transparent text-white p-10" use:inview on:enter on:leave>
	<div class="w-full h-96">
		<InformationCard {title} {msg} />
	</div>
</div>
