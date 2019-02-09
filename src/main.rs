extern crate serde;
extern crate serde_json;

extern crate walkdir;
extern crate scraper;

use std::fs::File;
use std::ffi::OsStr;
use std::path::Path;
use std::io::prelude::*;
use scraper::{Html, Selector};
use walkdir::WalkDir;

use serde_derive::{Serialize};
use serde_json::json;



#[derive(Serialize)]
struct Item {
    title: String,
    href: String,
    content: String,
}

static SEARCH_DIR: &str = "/home/sean/.notes/build/";

// inspired by https://gist.github.com/sebz/efddfc8fdcb6b480f567
fn parse_file(path: &Path) -> Item {
    let mut f = File::open(&path).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("something went wrong reading the file");
    let doc = Html::parse_document(&contents);
    let root_ref = doc.root_element();
    // TODO: blow up on non found titles..
 
    let title_el = root_ref.select(&Selector::parse("title").expect("a title")).next();
    let href = path.strip_prefix(SEARCH_DIR).unwrap();
    let title_str = title_el.map(|e| e.inner_html()).unwrap();
    let selector = Selector::parse("body").unwrap();
    let body = root_ref.select(&selector).next().unwrap();
    let text = body.text().collect::<Vec<&str>>();
    Item {
        title: title_str,
        href: String::from(href.to_str().unwrap()),
        content: text.join(" "),
    }
}

fn main() {
     let items: Vec<Item> = WalkDir::new(SEARCH_DIR)
        .into_iter()
        .filter_map(|v| v.ok())
        .filter(|dir_entry| {
            let meta = dir_entry.metadata().unwrap();
            if !meta.is_file(){
                return false;
            }
            let path = dir_entry.path().clone();
            let path_ext = path.extension().clone();
            match path_ext.unwrap_or(OsStr::new("foo")).to_str() {
                Some("html") => true,
                _ => false
            }
        })
        .map(|e| parse_file(e.path()))
        .collect();

    let json_items = json!(items).to_string();

    let mut write_file = File::create("lunr_index.json").unwrap();
    write_file.write_all(json_items.as_bytes()).unwrap();
}
