use crate::main;
use crate::other;
use crate::other::Song;
use crate::state;
use crate::state::Play;
use crate::state::PlayState;
use crate::GlobalApp;
use crossbeam::{
  atomic::AtomicCell,
  channel::{unbounded, Receiver, Sender},
  queue::ArrayQueue,
  sync::WaitGroup,
};
use hls_m3u8::{MasterPlaylist, MediaPlaylist, MediaSegment};
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use rand::seq::SliceRandom;
use rand::Rng;
use reqwest::header::ACCEPT;
use reqwest::header::RANGE;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::Response;
use rodio::{Decoder, Sink};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::ffi::OsString;
use std::fs::DirEntry;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::process::Command;
use std::str;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::command;
use tauri::Invoke;
use tauri::Manager;
use tauri::Wry;
use tauri_plugin_store::with_store;
use tauri_plugin_store::Store;
use tauri_plugin_store::StoreCollection;
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
pub struct ResponseBody {
  pub code: String,
  pub message: String,
  pub response: String,
  pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[command]
pub fn toggle_hls() {
  let a = GlobalApp::global();
  let b: tauri::State<'_, PlayState> = a.app.state();
  let mut c = b.0.lock().unwrap();
  c.toggle_hls()
}

#[command]
pub fn get_port() -> u16 {
  let a = GlobalApp::global();
  let b: tauri::State<'_, PlayState> = a.app.state();
  let c = b.0.lock().unwrap();
  c.get_port()
}

#[command]
pub fn hello_world_test(event: String) -> Option<String> {
  let stdout = hello_world(event);
  let a = GlobalApp::global();
  let b: tauri::State<'_, PlayState> = a.app.state();
  let c = b.0.lock().unwrap();
  let answer = c.get_title();
  let mut possible: Vec<String> = vec![];
  possible.push(stdout.to_lowercase().replace('\n', ""));
  possible.push(stdout.to_lowercase().replace('\n', "").replace(" ", ""));
  match stdout.to_lowercase().split_once('(') {
    Some((key, value)) => {
      possible.push(key.to_string());
      possible.push(key.to_string().replace(" ", ""));
    }
    None => (),
  };
  for d in answer {
    if possible.contains(&d) {
      let _ = a.app.emit_all("correct", true);
      return Some(stdout);
    }
  }

  Some(stdout)
}

#[command]
pub fn ls_test(event: String) -> Option<String> {
  let stdout = ls(event);
  Some(stdout)
}

#[command]
pub async fn open_login() {
  let a = GlobalApp::global();
  let b: tauri::State<'_, PlayState> = a.app.state();
  let c = b.0.lock().unwrap();
  let port = c.get_port();

  let url = format!("http://localhost:{}", port);
  let code = format!(
    " const MelonAPI = {{
      login: async (info) => {{
        fetch('{}', {{
        method: 'POST',
        headers: {{
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '{}',
        }},
        body: JSON.stringify(info)}})
        }},
      }}",
    url, url
  );

  let file_path = "../../src/path/login.html";

  let docs_window = tauri::WindowBuilder::new(
    &a.app,
    "melonLoginPopup",
    tauri::WindowUrl::App(file_path.into()),
  )
  .initialization_script(include_str!("../../src/path/preload.js"))
  .on_web_resource_request(|request, response| {
    if request.uri().starts_with("tauri://") {
      // if we have a CSP header, Tauri is loading an HTML file
      //  for this example, let's dynamically change the CSP
      if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
        println!("dd");
      }
    }
  })
  .build()
  .unwrap();
}

pub fn hello_world(event: String) -> String {
  let output = if cfg!(target_os = "windows") {
    Command::new("cmd")
      .args(["/C", format!("echo {}", event.to_string()).as_str()])
      .output()
      .expect("failed to execute process")
  } else {
    Command::new("sh")
      .arg("-c")
      .arg(format!("echo {}", event.to_string()).as_str())
      .output()
      .expect("failed to execute process")
  };
  if cfg!(target_os = "windows") {
    let res = event.clone();
    let v: Vec<u16> = event.encode_utf16().collect();
    let a = String::from_utf16(&v);
    let b = String::from_utf16_lossy(&v);
    let d = OsString::from(event);

    println!("{:#?}", v);
    println!("{:#?}", a);
    println!("{:#?}", b);
    println!("{:#?}", d);

    println!("{:#?}", output.stdout);

    return res;
  } else {
    let s = String::from_utf8(output.stdout);
    match s {
      Ok(s) => return s.replace("\r", "").replace("\n", ""),
      Err(e) => {
        println!("{e}");
        return String::new();
      }
    }
  }
}

pub fn ls(event: String) -> String {
  let output = Command::new("ls")
    .output()
    .expect("failed to execute process");

  print!("event: {}", event);
  let stdout = String::from_utf8(output.stdout).unwrap();
  return stdout;
}

#[command]
pub fn play() {
  thread::spawn(move || {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink: Sink = rodio::Sink::try_new(&handle).unwrap();

    let file: File = std::fs::File::open("./assets/NewJeans-01-Attention.mp3").unwrap();
    let buf: BufReader<File> = BufReader::new(file);
    let decoder: Decoder<BufReader<File>> = Decoder::new(buf).unwrap();

    sink.append(decoder);
    sink.set_volume(0.25);
    sink.sleep_until_end();
  });
}

#[tokio::main]
async fn get_list(argument: String) -> std::result::Result<serde_json::Value, reqwest::Error> {
  // lib
}

#[tokio::main]
pub async fn get_path(
  argument: String,
  hls: bool,
  token: String,
  session_id: String,
) -> std::result::Result<serde_json::Value, reqwest::Error> {
  // lib
}

#[tokio::main]
pub async fn get_mcache(
  argument: &String,
  size: u32,
) -> std::result::Result<(HeaderMap, Bytes), reqwest::Error> {
  let url = format!("{}", argument);
  let mut headers = HeaderMap::new();
  headers.insert(
    reqwest::header::ACCEPT,
    reqwest::header::HeaderValue::from_static("*/*"),
  );

  let client = reqwest::Client::new();
  let response = client.get(url).headers(headers.clone()).send().await?;

  let response_headers = response.headers().clone();
  let response_bytes = response.bytes().await?;

  Ok((response_headers, response_bytes))
}

#[tokio::main]
pub async fn get_ts(argument: &String) -> std::result::Result<String, reqwest::Error> {
  let url = format!("{}", argument);
  let response = reqwest::get(url).await.unwrap().text().await;

  response
}

fn test_trigger() {
  let a = GlobalApp::global();
  a.app
    .trigger_global("event-name", Some(format!("rs: abc")).into());
}

#[command]
pub fn simple_command_with_result(state: tauri::State<PlayState>, argument: String) -> Vec<Song> {
  let mut v: Vec<Song> = Vec::new();

  let result = get_list(argument);
  if let Ok(res) = result {
    let a = res.get("response").unwrap();
    let b = a.get("chartList").unwrap();
    let c = b.as_array().unwrap();
    let iter = c.iter();
    for i in iter {
      let artist_iter = i.get("artistList").unwrap().as_array().unwrap().iter();
      let mut artist_name: String = Default::default();
      for j in artist_iter {
        let name = j.get("artistName").unwrap().to_string().replace("\"", "");
        if artist_name.is_empty() {
          artist_name.push_str(&name);
        } else {
          artist_name.push_str(", ");
          artist_name.push_str(&name);
        }
      }
      v.push(Song {
        id: i.get("songId").unwrap().to_string().replace("\"", ""),
        title: i.get("songName").unwrap().to_string().replace("\"", ""),
        artist: artist_name,
        album: i.get("albumName").unwrap().to_string().replace("\"", ""),
        filename: "".to_string().replace("\"", ""),
        img: i.get("albumImgUrl").unwrap().to_string().replace("\"", ""),
        duration: i.get("playTime").unwrap().to_string().replace("\"", ""),
      });
    }
  }

  v.shuffle(&mut rand::thread_rng());
  let vv = v.to_vec();
  let a = Arc::clone(&state.0);
  let mut b = a.lock().unwrap();
  (*b).set_list(vv);
  (*b).play();

  v
}

#[command]
pub fn next(state: tauri::State<PlayState>) {
  let b = Arc::clone(&state.0);
  let mut c = b.lock().unwrap();
  (*c).next();
}

#[command]
pub fn prev(state: tauri::State<PlayState>) {
  let b = Arc::clone(&state.0);
  let mut c = b.lock().unwrap();
  (*c).prev();
}

#[command]
pub fn play_or_pause(state: tauri::State<PlayState>) {
  let b = Arc::clone(&state.0);
  let mut c = b.lock().unwrap();
  (*c).play_or_pause();
}

#[command]
pub fn simple_test(state: tauri::State<PlayState>, argument: String) -> Vec<u8> {
  let s = Arc::clone(&state.0);
  let b = Arc::clone(&state.0);
  let mut c = b.lock().unwrap();
  (*c).increase();
  let count = (*c).get_count();
  thread::spawn(move || other::read_audio2(Arc::clone(&s), argument.to_string(), count));
  Vec::new()
}

#[command]
pub fn read_files() -> Vec<Song> {
  other::read_files()
}
