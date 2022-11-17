<script lang="ts">
	import { fade } from 'svelte/transition';
</script>

<div class="flex flex-col items-center justify-center w-screen h-screen bg-black z-50" out:fade>
	<img src="https://media.giphy.com/media/B8dUiWJmQi4KCQhS9x/giphy.gif" alt="Loading GIF" />
	<span class="text-white font-merriweather text-xl"> Cargando... </span>
</div>
