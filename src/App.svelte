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
          return { name: f, stored: false };
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
    fields.push({ name: "", stored: false });
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
</script>

<header>
  {#each fields as field}
    <input type="text" bind:value={field.name} />
    <input type="checkbox" bind:checked={field.stored} />
  {/each}
  <button on:click={field_push}>+</button>
</header>
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
