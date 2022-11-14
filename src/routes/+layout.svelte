<script>
	import '../app.css';
</script>

<div class="flex flex-col w-full h-full font-raleway">
	<div class="z-50 flex w-full h-16 items-center justify-end text-white cristal">
		<nav class="flex flex-wrap items-center justify-evenly bg-transparent w-1/2 h-full text-xl">
			<a href="/">Inicio</a>
			<a href="/">Sobre mí</a>
			<a href="/">Proyectos</a>
			<a href="/">Contacto</a>
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
