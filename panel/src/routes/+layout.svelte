<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css'
	import Sidebar from '$lib/components/Sidebar.svelte';

	let { children } = $props();
	let sidebarHover = $state(false);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<Sidebar bind:hover={sidebarHover}>
	<nav>
		<span>Navigation</span>
	</nav>
</Sidebar>

<main class="main-content" style="padding-left: {sidebarHover ? '250px' : '20px'}">
	{@render children()}
</main>

<style>
	.main-content {
		transition: padding-left 0.3s ease;
		min-height: 100vh;
	}
</style>
