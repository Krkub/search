<script lang="ts">
  import { event, invoke } from "@tauri-apps/api";
  import { onMount } from "svelte";
  import Schema from "./lib/Schema.svelte";
  import Data from "./lib/Data.svelte";
  import SearchBar from "./lib/Bar.svelte";

  let page = 1;
  let page_len = 100;
  $: start = (page - 1) * page_len;
  $: end = page * page_len;
  let len = 0;
  $: pages = Math.ceil(len / page_len);
  let data: Object[] = [];
  let schema = [""];
  let indexing: Promise<unknown>;
  let gen = 1_000_0;
  onMount(async () => {
    ({ data, len } = await invoke<{ data: Object[]; len: number }>(
      "fetch_data",
      { start, end }
    ));
    schema = Object.keys(data[0]);
  });
  event.listen("data_mutate", async (e) => {
    console.log(e);
    ({ data, len } = await invoke<{ data: Object[]; len: number }>(
      "fetch_data",
      { start, end }
    ));
    schema = Object.keys(data[0]);
    event.emit("index");
  });
  event.listen("index", (e) => {
    console.log("ev");
    indexing = invoke("index_data", { head: schema });
  });
</script>

<main>
  <SearchBar /> <br />
  <button
    on:click={async () => {
      schema = ["first_name", "last_name", "job", "bio", "company"];
      await invoke("add_fake", { len: gen });
      console.log("dez");
      event.emit("data_mutate","faker");
    }}>add</button
  >
  <input type="number" bind:value={gen} />
  <Schema bind:schema /><br />
  <Data bind:data {schema} />
</main>

<style>
  :global(*) {
    color: white;
    background-color: black;
  }
</style>
