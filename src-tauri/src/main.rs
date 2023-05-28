// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fake::faker;
use fake::Fake;
use std::{collections::HashMap, fs::File, sync::Mutex};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Document, Schema, STORED, TEXT},
    Index, ReloadPolicy,
};
use tauri::{
    api::{dialog::FileDialogBuilder, file},
    generate_handler, CustomMenuItem, Manager, Menu, Submenu, WindowMenuEvent,
};
fn main() {
    tauri::Builder::default()
        .manage(MyState::default())
        .menu(
            Menu::os_default("google_v2").add_submenu(Submenu::new(
                "File",
                Menu::new()
                    .add_item(CustomMenuItem::new("open", "Open").accelerator("cmdOrControl+O"))
                    .add_item(CustomMenuItem::new("save", "Save").accelerator("cmdOrControl+S")), //.add_item(CustomMenuItem::new("quit", "Quit").accelerator("cmdOrControl+Q")),
            )),
        )
        .on_menu_event(os_menu_handler)
        .invoke_handler(generate_handler![
            index_data,
            search_index,
            fetch_data,
            mutate_data,
            add_fake
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
fn os_menu_handler(event: WindowMenuEvent) {
    match event.menu_item_id() {
        //"quit" => event.window().close().unwrap(),
        "save" => FileDialogBuilder::new()
            .add_filter("json", &["json"])
            .add_filter("all files", &["*"])
            .set_title("export")
            .save_file(move |file_path| match file_path {
                Some(e) => {
                    let ev = event;
                    let win = ev.window();
                    let st = win.state::<MyState>();
                    let data = st.data.lock().unwrap();
                    let file = File::create(e).unwrap();
                    serde_json::to_writer(file, &*data).unwrap();
                }
                None => {}
            }),
        "open" => FileDialogBuilder::new()
            .add_filter("json", &["json"])
            .add_filter("all files", &["*"])
            .set_title("import")
            .pick_file(move |file_path| match file_path {
                Some(e) => {
                    let ev = event;
                    let win = ev.window();
                    let st = win.state::<MyState>();
                    let mut data = st.data.lock().unwrap();

                    *data = serde_json::from_str(&file::read_string(e).unwrap()).unwrap();
                    win.emit("data_mutate", "mutate").unwrap();
                }
                None => {}
            }),
        _ => {}
    };
}
#[derive(Default)]
struct MyState {
    data: Mutex<Vec<HashMap<String, String>>>,
    index: Mutex<Option<Index>>,
    schema: Mutex<Option<Schema>>,
}
// remember to call `.manage(MyState::default())`
#[tauri::command]
fn index_data(state: tauri::State<'_, MyState>, head: Vec<String>) -> Result<(), String> {
    println!("indexing");
    let body = state.data.lock().unwrap();
    let mut schema_builder = Schema::builder();
    for e in head.iter().filter(|e| !e.is_empty()) {
        schema_builder.add_text_field(&e, TEXT | STORED);
    }
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    let mut index_writer = index.writer(2_000_000_000).unwrap();
    for e in &*body {
        let mut doc = Document::new();
        for (i, f) in e {
            doc.add_text(schema.get_field(&i).expect("indexing error"), f)
        }
        index_writer.add_document(doc).expect("indexing arror");
    }
    index_writer.commit().expect("indaxing err");
    println!("index finish");
    let mut s_schema = state.schema.lock().unwrap();
    *s_schema = Some(schema);
    let mut s_index = state.index.lock().unwrap();
    *s_index = Some(index);
    Ok(())
}

#[tauri::command]
fn search_index(
    state: tauri::State<'_, MyState>,
    query: String,
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
            let query_b = query_parser
                .parse_query(&query)
                .expect("qery parsing error");
            let top_docs = searcher
                .search(&query_b, &TopDocs::with_limit(limit))
                .unwrap();
            let mut res: Vec<SearchData> = Vec::new();
            for (score, doc_address) in top_docs {
                res.push(SearchData {
                    values: searcher.doc(doc_address).expect("something went wrong"),
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
    values: Document,
    score: f32,
}

#[tauri::command]
async fn fetch_data(
    state: tauri::State<'_, MyState>,
    start: usize,
    end: usize,
) -> Result<FetchRet, String> {
    let data = state.data.lock().unwrap();
    let len = data.len();
    let slice =
        data[(if start > len { 0 } else { start })..(if end > len { len } else { end })].to_vec();

    Ok(FetchRet {
        data: slice,
        len: len,
    })
}
#[tauri::command]
async fn mutate_data(
    state: tauri::State<'_, MyState>,
    index: usize,
    new_data: HashMap<String, String>,
) -> Result<(), String> {
    let mut data = state.data.lock().unwrap();
    match data.get_mut(index) {
        Some(e) => *e = new_data,
        None => data.insert(index, new_data),
    }
    // println!("{:?}", data);
    Ok(())
}
#[derive(serde::Serialize)]
struct FetchRet {
    data: Vec<HashMap<String, String>>,
    len: usize,
}
#[tauri::command]
async fn add_fake(state: tauri::State<'_, MyState>, len: usize) -> Result<(), String> {
    let mut arr = state.data.lock().unwrap();
    for i in 0..len {
        println!("{i}");
        arr.push(
            [
                (
                    "first_name".to_string(),
                    faker::name::en::FirstName().fake::<String>(),
                ),
                (
                    "last_name".to_string(),
                    faker::name::en::LastName().fake::<String>(),
                ),
                ("job".to_string(), faker::name::en::Title().fake::<String>()),
                (
                    "bio".to_string(),
                    faker::lorem::en::Paragraph(1..4).fake::<String>(),
                ),
                (
                    "company".to_string(),
                    faker::company::en::CompanyName().fake::<String>(),
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
    println!("works");
    Ok(())
}
