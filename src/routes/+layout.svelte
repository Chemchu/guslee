<script>
	import '../app.css';
</script>

<div class="flex flex-col w-full h-full font-raleway">
	<div class="fixed z-50 flex w-full h-10 md:h-16 items-center justify-end text-white cristal">
		<nav
			class="flex flex-wrap items-center justify-evenly w-full md:w-2/5 h-full text-sm md:text-xl"
		>
			<a class="hover:scale-110 duration-200 hover:text-orange-500 hover:font-semibold" href="/"
				>Inicio</a
			>
			<a class="hover:scale-110 duration-200 hover:text-orange-500 hover:font-semibold" href="/"
				>Sobre mí</a
			>
			<a class="hover:scale-110 duration-200 hover:text-orange-500 hover:font-semibold" href="/"
				>Proyectos</a
			>
		</nav>
	</div>
	<slot />
</div>

<style>
	.cristal {
		background: linear-gradient(135deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0));
		backdrop-filter: blur(10px);
		-webkit-backdrop-filter: blur(10px);
		border: 1px solid rgba(255, 255, 255, 0.18);
		box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
	}
</style>
