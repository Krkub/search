<script lang="ts">
    import { invoke } from "@tauri-apps/api";
    let query: string;
    let limit: number = 10;
    let res: Promise<QueryRes[]> | undefined;
    function doQuyery() {
        res = invoke<QueryRes[]>("search_index", { query, limit });
    }
    interface QueryRes {
        score: number;
        values: Values;
    }

    interface Values {
        field_values: FieldValue[];
    }

    interface FieldValue {
        field: number;
        value: string;
    }
</script>

<main>
    <input type="text" bind:value={query} placeholder="put your query here" />
    <input type="number" min="1" bind:value={limit} placeholder="limit" />
    <button on:click={doQuyery}>Go!</button> <br>
    {#if res}
        {#await res}
            <p>searching</p>
        {:then resu}
            <!-- {JSON.stringify(resu)} -->
            {#each resu as resul}
                {#each resul.values.field_values as result}
                    {result.field}:{result.value}  <br>
                {/each}
                {resul.score} <hr>
            {/each}
        {/await}
    {/if}
</main>
