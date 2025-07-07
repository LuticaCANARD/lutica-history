<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>Nameplate of Lutica</title>
	<meta name="description" content="Lutica's personal introduction page, showcasing skills and interests.">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="icon" href="/favicon.ico" type="image/x-icon">
	<link rel="stylesheet" href="/global.css">
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;700&display=swap" rel="stylesheet">
</svelte:head>
{@render children()}
