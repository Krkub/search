// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, sync::Mutex};

use serde::Deserialize;
use tantivy::{doc, schema::*, Index};
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
        .invoke_handler(generate_handler![index_data])
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
    let mut s_index = state.index.lock().unwrap();
    *s_index = Some(index);
    Ok(())
}
