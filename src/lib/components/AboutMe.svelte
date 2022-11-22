<script>
	import InformationCard from './cards/InformationCard.svelte';

	export let id = 'id';
	const title = 'Un poco de mí';
	const msg = `Me considero una persona sencilla y agradable.
	He cursado el grado en ingeniería informática y también el máster en ingeniería y tecnología de sistemas software (MITSS) en la Universitat Politècnica de València.
	Soy fan de las guitarras, el café y el diseño web.`;
</script>

<div {id} class="flex items-center w-screen min-h-screen text-white p-10">
	<div class="w-full h-96">
		<InformationCard {title} {msg} />
	</div>
</div>
