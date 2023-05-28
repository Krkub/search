<script lang="ts">
    export let schema: string[];
</script>

<main>
    {#each schema as field}
        <input type="text" bind:value={field} />
    {/each}
    <button
        on:click={() => {
            schema.push("");
            schema = schema;
        }}>+</button
    >
</main>
