// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, sync::Mutex};

use serde::Deserialize;
use tantivy::{collector::TopDocs, doc, query::QueryParser, schema::*, Index, ReloadPolicy};
use tauri::{generate_handler, CustomMenuItem, Menu, Submenu, WindowMenuEvent};
fn main() {
    tauri::Builder::default()
        .menu(
            Menu::os_default("google_v2").add_submenu(Submenu::new(
                "File",
                Menu::new()
                    .add_item(CustomMenuItem::new("open", "Open").accelerator("cmdOrControl+O"))
                    .add_item(CustomMenuItem::new("save", "Save").accelerator("cmdOrControl+S")), //.add_item(CustomMenuItem::new("quit", "Quit").accelerator("cmdOrControl+Q")),
            )),
        )
        .on_menu_event(os_menu_handler)
        .manage(MyState::default())
        .invoke_handler(generate_handler![index_data, search_index])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
fn os_menu_handler(event: WindowMenuEvent) {
    println!("{:?}", event.menu_item_id());
    match event.menu_item_id() {
        //"quit" => event.window().close().unwrap(),
        "save" => event.window().emit("menu-event", "save-event").unwrap(),
        "open" => event.window().emit("menu-event", "open-event").unwrap(),
        _ => {}
    };
}
#[derive(Deserialize)]
struct HeadData {
    name: String,
    stored: bool,
}
#[derive(Default)]
struct MyState {
    //head: Mutex<Vec<HeadData>>,
    index: Mutex<Option<Index>>,
    schema: Mutex<Option<Schema>>,
}
// remember to call `.manage(MyState::default())`
#[tauri::command]
fn index_data(
    state: tauri::State<'_, MyState>,
    head: Vec<HeadData>,
    body: Vec<HashMap<String, String>>,
) -> Result<(), String> {
    let mut schema_builder = Schema::builder();
    for e in head.iter().filter(|e| !e.name.is_empty()) {
        schema_builder.add_text_field(&e.name, if e.stored { TEXT | STORED } else { TEXT });
    }
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    let mut index_writer = index.writer(100_000_000).unwrap();
    for e in body {
        let mut doc = Document::new();
        for (i, f) in e {
            doc.add_text(schema.get_field(&i).expect("indexing error"), f)
        }
        index_writer.add_document(doc).expect("indexing arror");
    }
    index_writer.commit().expect("indaxing err");
    let mut s_schema = state.schema.lock().unwrap();
    *s_schema = Some(schema);
    let mut s_index = state.index.lock().unwrap();
    *s_index = Some(index);
    Ok(())
}
// remember to call `.manage(MyState::default())`
#[tauri::command]
fn search_index(
    state: tauri::State<'_, MyState>,
    query_s: String,
    limit: usize,
) -> Result<Vec<SearchData>, String> {
    let index_m = (*state.index.lock().unwrap()).clone();
    match index_m {
        Some(e) => {
            let reader = e
                .reader_builder()
                .reload_policy(ReloadPolicy::OnCommit)
                .try_into()
                .expect("something went wrong");
            let schema_m = (*state.schema.lock().unwrap()).clone();
            let searcher = reader.searcher();
            let query_parser = QueryParser::for_index(
                &e,
                schema_m
                    .expect("no_schema")
                    .fields()
                    .map(|f| {
                        let (field, _) = f;
                        field
                    })
                    .collect(),
            );
            let query = query_parser
                .parse_query(&query_s)
                .expect("qery parsing error");
            let top_docs = searcher
                .search(&query, &TopDocs::with_limit(limit))
                .unwrap();
            let mut res: Vec<SearchData> = Vec::new();
            for (score, doc_address) in top_docs {
                res.push(SearchData {
                    field_values: searcher.doc(doc_address).expect("something went wrong"),
                    score,
                });
            }
            Ok(res)
        }
        None => Err("no index".to_string()),
    }
}

#[derive(serde::Serialize)]
struct SearchData {
    field_values: Document,
    score: f32,
}
