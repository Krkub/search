<script lang="ts">
    import { invoke, event } from "@tauri-apps/api";
    export let index: number;
    export let schema: string[];
    export let data: Object;
    let saving: Promise<unknown>;
    function mu() {
        saving = invoke("mutate_data", { index, newData: data });
        event.emit("index","SingleData");
    }
</script>

<main>
    {#each schema as field}
        <input
            type="text"
            bind:value={data[field]}
            placeholder={field}
            on:change={mu}
        />
    {/each}
</main>
