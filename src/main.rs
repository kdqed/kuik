use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use handlebars::Handlebars;
use pulldown_cmark::{Parser, Options, html};
use std::env;
use std::process;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

fn render_md(path: PathBuf) -> String {
  let str_path: String = path.to_str().unwrap().to_string();
  let mut file = File::open(str_path).unwrap();
  let mut contents = String::new();
  file.read_to_string(&mut contents).unwrap();
  let split = contents.split("---");
  let chunks: Vec<&str> = split.take(3).collect();

  let mut yml_header: BTreeMap<String, String> = serde_yaml::from_str(&chunks[1]).unwrap();
    
  let mut rendered_md = String::new();
  let mut options = Options::empty();
  options.insert(Options::ENABLE_STRIKETHROUGH);
  let parser = Parser::new_ext(chunks[2], options);
  html::push_html(&mut rendered_md, parser);
  yml_header.insert("rendered_md".to_string(),rendered_md);
    
  let mut template_file = File::open("templates/".to_owned()+&yml_header["template"]).unwrap();
  let mut template = String::new();
  template_file.read_to_string(&mut template).unwrap();
  let handlebars = Handlebars::new();
  return handlebars.render_template(&template,&yml_header).unwrap();
     
}

async fn index(req: HttpRequest) -> Result<NamedFile> {
  let mut path: PathBuf = PathBuf::new();
  path.push("site");
  
  let req_path: PathBuf = req.match_info().query("filename").parse().unwrap();
  path.push(req_path);
  
  let mut str_path: String = path.to_str().unwrap().to_string();
  
  if str_path.ends_with("/") {
    str_path.pop();
    path = PathBuf::new();
    path.push(str_path);
  }
  
  let mut ext = path.extension().unwrap_or(OsStr::new(""));
  
  if ((!path.exists()) || (path.is_dir())) && ext!="html" {
    path.push("index.html");
  }
  
  ext = path.extension().unwrap_or(OsStr::new(""));
  if ext=="html" {
    path.set_extension("md");
    if !path.exists() {
      path.set_extension("html");
    }
    ext = path.extension().unwrap_or(OsStr::new(""));
  }
  
  if path.exists() && ext=="md" {
    let contents = render_md(path);
    let mut tmp_file = File::create(".temp.html")?;
    tmp_file.write_all(contents.as_bytes())?;
    Ok(NamedFile::open(".temp.html")?)
  }
  else {
    Ok(NamedFile::open(path)?)
  }
}

fn build(test: bool) {
  for entry in WalkDir::new("./site").into_iter().filter_map(|e| e.ok()) {
    let f_name = entry.file_name().to_string_lossy();
    if f_name.ends_with(".md") {
      let mut path_buf = entry.path().to_path_buf();
      path_buf.set_extension("html");
      let str_path: String = path_buf.to_str().unwrap().to_string();
      println!("Generating: {}", str_path);
      let path = entry.path();
      let rendered_content = render_md(path.to_path_buf());
      if !test {
        let mut dest_file = File::create(str_path).unwrap();
        dest_file.write_all(("<!-- built by gen -->\n".to_owned()+&rendered_content).as_bytes()).unwrap();
      }
    }
  }
}

fn help() {
  println!("gen v0.1: Converts all .md files in site/ into .html");
  println!("\nUsage:\n");
  println!("1. For dev server: \ngen --serve\n");
  println!("2. For build: \ngen --build\n");
  println!("3. For test build (run check without generating any files): \ngen --test");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  
  if args.len()<2 {
    help();
    process::exit(1);
  } 
  else if args[1]=="--serve" {  
    println!("Starting Server...");
  }
  else if args[1]=="--build" {
    build(false); // test=false
    process::exit(1);
  }
  else if args[1]=="--test" {
    build(true); // test=true
    process::exit(1);
  }
  else{
    help();
    process::exit(1);
  }
  
  
  use actix_web::{web, App, HttpServer};
  HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(index)))
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
 
