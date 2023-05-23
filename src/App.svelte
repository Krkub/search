<script lang="ts" type="module">
  import { event, dialog, fs, invoke } from "@tauri-apps/api";
  event.listen<string>("menu-event", async (e) => {
    console.log(e);
    if (e.payload === "open-event") {
      let path = await dialog.open({
        multiple: false,
        filters: [{ extensions: ["json"], name: "data" }],
      });
      if (!Array.isArray(path) && path) {
        let file = await fs.readTextFile(path);
        data = JSON.parse(file);
        fields = Object.keys(data[0]).map((f) => {
          return { name: f, stored: true };
        });
      }
    } else if (e.payload === "save-event") {
      let path = await dialog.save({
        filters: [{ name: "data", extensions: ["json"] }],
      });
      if (path) {
        await fs.writeTextFile(path, JSON.stringify(data));
      }
    }
  });
  let fields: HeadData[] = [];
  function field_push() {
    fields.push({ name: "", stored: true });
    fields = fields;
  }
  let data: Object[] = [];
  function map_push() {
    data.push(new Object());
    data = data;
  }
  $: indexing = invoke("index_data", { head: fields, body: data });
  interface HeadData {
    name: string;
    stored: boolean;
  }

  let query = "query";
  let limit = 10;
  let result: Promise<
    {
      field_values: { field_values: { field: number; value: string }[] };
      score: number;
    }[]
  >;
  $: console.log(result);
</script>

<header>
  <input type="text" bind:value={query} />
  <input type="number" bind:value={limit} />
  <button
    on:click={() =>
      (result = invoke("search_index", { queryS: query, limit: limit }))}
    >search</button
  >
</header>
<div>
  {#await result}
    Searching...
  {:then res}
    {#if res}
      {#each res as field}
      {#each field.field_values.field_values as data}
        {data.value} <br>
      {/each}
        {field.score}
      {/each}
      <p>found</p>
    {/if}
  {/await}
</div>
<section>
  {#each fields as field}
    <input type="text" bind:value={field.name} />
    <input type="checkbox" bind:checked={field.stored} />
  {/each}
  <button on:click={field_push}>+</button>
</section>
{#await indexing}
  <p>Laoding..</p>
{:then res}
  <p>{res}</p>
{/await}
<main>
  {#each data as item}
    {#each fields as field}
      <input
        type="text"
        bind:value={item[field.name]}
        placeholder={field.name}
      />
    {/each}
  {/each}
  <button on:click={map_push}>+</button>
</main>

<style>
  :global(*) {
    color: white;
    background-color: black;
  }
</style>
