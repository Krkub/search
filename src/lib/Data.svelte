<script lang="ts">
    import SingleData from "./SingleData.svelte";
    export let schema: string[];
    export let data: Object[];
</script>

<main>
    {#each data as field, index}
        <SingleData {index} {schema} bind:data={field} />
    {/each}
    <button
        on:click={() => {
            data.push({});
            data = data;
        }}>+</button
    >
</main>
