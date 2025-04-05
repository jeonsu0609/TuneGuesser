// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
// mod decode;
// mod mcache;
mod other;
mod output;
mod resampler;
mod state;

use std::{
  io::{Read, Write},
  net::{SocketAddr, TcpListener, TcpStream},
  path::PathBuf,
  sync::{Arc, Mutex},
  thread,
};

// use crossbeam::channel::unbounded;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::json;
use state::{Play, PlayState};
use tauri::{
  api::dialog::ask, http::ResponseBuilder, ipc::RemoteDomainAccessScope, utils::config::AppUrl,
  AppHandle, CustomMenuItem, GlobalShortcutManager, LogicalSize, Manager, Menu, MenuItem, RunEvent,
  Submenu, Window, WindowBuilder, WindowEvent, WindowUrl,
};
use tauri_plugin_oauth::start;
use tauri_plugin_store::{with_store, StoreBuilder};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug)]
pub struct GlobalApp {
  app: AppHandle,
}
static INSTANCE: OnceCell<GlobalApp> = OnceCell::new();

impl GlobalApp {
  pub fn global() -> &'static GlobalApp {
    INSTANCE.get().expect("App is not initialized")
  }

  pub fn new(app: AppHandle) -> GlobalApp {
    GlobalApp { app }
  }
}

#[derive(Serialize)]
struct Reply {
  data: String,
}

#[derive(Serialize, Deserialize)]
struct HttpPost {
  foo: String,
  bar: String,
}

#[derive(Serialize)]
struct HttpReply {
  msg: String,
  request: HttpPost,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
async fn menu_toggle(window: tauri::Window) {
  window.menu_handle().toggle().unwrap();
}

fn rs2js<R: tauri::Runtime>(message: Payload, manager: &impl Manager<R>) {
  manager.emit_all("rs2js", format!("rs: abc")).unwrap();
}

pub fn start_with_config<F: FnMut(String) + Send + 'static>(
  mut handler: F,
) -> Result<u16, std::io::Error> {
  let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 12345)))?;
  let port = listener.local_addr()?.port();

  thread::spawn(move || {
    for conn in listener.incoming() {
      match conn {
        Ok(conn) => {
          if let Some(url) = handle_connection(conn, None, port) {
            // Using an empty string to communicate that a shutdown was requested.
            if !url.is_empty() {
              handler(url);
            }
            // TODO: Check if exiting here is always okay.
            break;
          }
        }
        Err(err) => {
          log::error!("Error reading incoming connection: {}", err.to_string());
        }
      }
    }
  });

  Ok(port)
}

async fn handle_connection_ws<'a>(raw_stream: tokio::net::TcpStream, addr: SocketAddr) {
  println!("Incoming TCP connection from: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(raw_stream)
    .await
    .expect("Error during the websocket handshake occurred");
  println!("WebSocket connection established: {}", addr);

  // Insert the write part of this peer to the peer map.
  let (tx, rx) = unbounded();

  let (outgoing, incoming) = ws_stream.split();

  let broadcast_incoming = incoming.try_for_each(|msg| {
    let text = msg.to_text().unwrap();
    println!("Received a message from {}: {}", addr, text);

    //sending response
    let response = format!("Rust received \"{}\"", text);
    tx.unbounded_send(response.as_str().into()).unwrap();

    future::ok(())
  });

  let receive_from_others = rx.map(Ok).forward(outgoing);

  pin_mut!(broadcast_incoming, receive_from_others);
  future::select(broadcast_incoming, receive_from_others).await;

  println!("{} disconnected", &addr);
}

fn handle_connection(mut conn: TcpStream, response: Option<&str>, port: u16) -> Option<String> {
  let mut buffer = [0; 4048];
  if let Err(io_err) = conn.read(&mut buffer) {
    log::error!("Error reading incoming connection: {}", io_err.to_string());
  };

  let mut headers = [httparse::EMPTY_HEADER; 16];
  let mut request = httparse::Request::new(&mut headers);

  // let m = request.parse(&buffer); //.ok()?;
  request.parse(&buffer).ok()?;

  let path = request.path.unwrap_or_default();
  if path == "/exit" {
    return Some(String::new());
  };

  let mut is_localhost = false;

  for header in &headers {
    if header.name == "Full-Url" {
      return Some(String::from_utf8_lossy(header.value).to_string());
    } else if header.name == "Host" {
      is_localhost = String::from_utf8_lossy(header.value).starts_with("localhost");
    }
  }
  if path == "/cb" {
    log::error!("Client fetched callback path but the request didn't contain the expected header.");
  }

  let script = format!(
    r#"<script>fetch("http://{}:{}/cb",{{headers:{{"Full-Url":window.location.href}}}})</script>"#,
    if is_localhost {
      "localhost"
    } else {
      "127.0.0.1"
    },
    port
  );
  let response = match response {
    Some(s) if s.contains("<head>") => s.replace("<head>", &format!("<head>{}", script)),
    Some(s) if s.contains("<body>") => {
      s.replace("<body>", &format!("<head>{}</head><body>", script))
    }
    Some(s) => {
      log::warn!(
        "`response` does not contain a body or head element. Prepending a head element..."
      );
      format!("<head>{}</head>{}", script, s)
    }
    None => format!(
      "<html><head>{}</head><body>Please return to the app.</body></html>",
      script
    ),
  };

  // TODO: Test if unwrapping here is safe (enough).
  conn
    .write_all(
      format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        response.len(),
        response
      )
      .as_bytes(),
    )
    .unwrap();
  conn.flush().unwrap();

  None
}

#[tauri::command]
async fn listen() {
  let server = tokio::net::TcpListener::bind("127.0.0.1:9001")
    .await
    .unwrap();

  while let Ok((stream, addr)) = server.accept().await {
    tokio::spawn(handle_connection_ws(stream, addr));
  }
}

#[tauri::command]
fn start_server(state: tauri::State<PlayState>, window: Window) -> Result<u16, String> {
  let port = start_with_config(move |url| {
    // Because of the unprotected localhost port, you must verify the URL here.
    // Preferebly send back only the token, or nothing at all if you can handle everything else in Rust.
    let _ = window.emit("redirect_uri", url.clone());
    println!("{}", url);
    let mut token = String::new();
    let mut session_id = String::new();
    let mut member_key = String::new();
    let a: Vec<&str> = url.split('?').collect();
    let b = a.get(1);

    match b {
      Some(p) => {
        let c: Vec<&str> = p.split('&').collect();
        for i in &c {
          let d: Vec<&str> = i.split('=').collect();
          let key = d.get(0).unwrap();
          let value = d.get(1).unwrap();

          if *key == "token" {
            token = value.to_string();
          }
          if *key == "sessionId" {
            session_id = value.to_string();
          }
          if *key == "memberKey" {
            member_key = value.to_string();
          }
        }
      }
      None => (),
    }

    std::thread::spawn(move || {
      let a = GlobalApp::global();
      a.app
        .clone()
        .get_window("melonLoginPopup")
        .unwrap()
        .close()
        .unwrap();
    });

    let a = GlobalApp::global();
    let _ = a.app.emit_all("login", member_key);

    let b: tauri::State<'_, PlayState> = a.app.state();
    let mut c = b.0.lock().unwrap();
    c.set_token(token);
    c.set_session_id(session_id);

    println!("{} {}", c.get_token(), c.get_session_id());
  })
  .map_err(|err| err.to_string());
  let a = Arc::clone(&state.0);
  let mut b = a.lock().unwrap();
  println!("{}", port.clone().unwrap());
  (*b).set_port(port.clone().unwrap());

  port
}

fn main() {
  #[allow(unused_mut)]
  tauri_plugin_deep_link::prepare("de.fabianlars.deep-link-test");
  let http = tauri_invoke_http::Invoke::new(["*"]);
  let mut app = tauri::Builder::default()
    .on_page_load(|window, _| {
      let window_ = window.clone();
      window.listen("js-event", move |event| {
        println!("got js-event with message '{:?}'", event.payload());
        let reply = Reply {
          data: "something else".to_string(),
        };
        window_
          .emit("rust-event", Some(&reply))
          .expect("failed to emit");
      });
      let _ = window.set_size(LogicalSize {
        width: 800,
        height: 800,
      });

      println!("{:#?}", window.url());
    })
    .register_uri_scheme_protocol("customprotocol", move |_app_handle, request| {
      if request.method() == "POST" {
        let request: HttpPost = serde_json::from_slice(request.body()).unwrap();
        return ResponseBuilder::new()
          .mimetype("application/json")
          .header("Access-Control-Allow-Origin", "*")
          .status(200)
          .body(serde_json::to_vec(&HttpReply {
            request,
            msg: "Hello from rust!".to_string(),
          })?);
      }
      ResponseBuilder::new()
        .mimetype("text/html")
        .status(404)
        .body(Vec::new())
    })
    .menu(get_menu())
    .on_menu_event(|event| {
      println!("{:?}", event.menu_item_id());
    })
    .invoke_handler(tauri::generate_handler![
      cmd::hello_world_test,
      cmd::ls_test,
      cmd::play,
      cmd::simple_command_with_result,
      cmd::simple_test,
      cmd::read_files,
      cmd::next,
      cmd::prev,
      cmd::play_or_pause,
      cmd::open_login,
      cmd::get_port,
      cmd::toggle_hls,
      menu_toggle,
      start_server,
      listen
    ])
    .plugin(tauri_plugin_store::Builder::default().build())
    .setup(move |app| {
      let handle = app.handle();
      let clone_handle = app.handle().clone();
      let id = app.listen_global("event-name", move |event| {
        // i think that this needs the move keyword, not sure right now
        println!("got event-name with payload {:?}", event.payload());
        rs2js(
          Payload {
            message: "Tauri is awesome!".into(),
          },
          &handle,
        );
      });

      let _: tauri::EventHandler = app.listen_global("MelonAPI", move |a| {
        // i think that this needs the move keyword, not sure right now
        println!("got event-name with payload {:?}", a.payload());
      });

      let _ = tauri_plugin_deep_link::register("myapp", |info| {
        println!("login");
      });

      // Don't unlisten here as it would make the listen_global call above useless (immediately unregistered)
      // app.unlisten(id);

      // app
      //   .emit_all(
      //     "event-name",
      //     Payload {
      //       message: "Tauri is awesome!".into(),
      //     },
      //   )
      //   .unwrap();

      // let _ = with_store(
      //   app.handle(),
      //   app.state(),
      //   PathBuf::from("settings.dat"),
      //   |store| store.insert("app".to_string(), json!("b")),
      // );
      let instance_app: GlobalApp = GlobalApp::new(app.handle());
      INSTANCE.set(instance_app).unwrap();

      let mut store = StoreBuilder::new(app.handle(), "setting.json".parse()?).build();
      let _ = store.insert("a".to_string(), json!("b"));

      app.manage(PlayState(Arc::new(Mutex::new(Play::new(clone_handle)))));

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  app.run(|app_handle, e| match e {
    // Application is ready (triggered only once)
    RunEvent::Ready => {
      let app_handle = app_handle.clone();
      app_handle
        .global_shortcut_manager()
        .register("CmdOrCtrl+1", move || {
          let app_handle = app_handle.clone();
          let window = app_handle.get_window("main").unwrap();
          window.set_title("New title!").unwrap();
        })
        .unwrap();
    }

    // Triggered when a window is trying to close
    RunEvent::WindowEvent {
      label,
      event: WindowEvent::CloseRequested { api, .. },
      ..
    } => {
      let app_handle = app_handle.clone();
      let window = app_handle.get_window(&label).unwrap();
      // use the exposed close api, and prevent the event loop to close
      api.prevent_close();
      // ask the user if he wants to quit
      ask(
        Some(&window),
        "Tauri API",
        "Are you sure that you want to close this window?",
        move |answer| {
          if answer {
            // .close() cannot be called on the main thread
            std::thread::spawn(move || {
              app_handle.get_window(&label).unwrap().close().unwrap();
            });
          }
        },
      );
    }

    // Keep the event loop running even if all windows are closed
    // This allow us to catch system tray events when there is no window
    RunEvent::ExitRequested { api, .. } => {
      api.prevent_exit();
    }
    _ => {}
  })
}

pub fn get_menu() -> Menu {
  #[allow(unused_mut)]
  let mut disable_item =
    CustomMenuItem::new("disable-menu", "Disable menu").accelerator("CmdOrControl+D");
  #[allow(unused_mut)]
  let mut test_item = CustomMenuItem::new("test", "Test").accelerator("CmdOrControl+T");
  #[cfg(target_os = "macos")]
  {
    disable_item = disable_item.native_image(tauri::NativeImage::MenuOnState);
    test_item = test_item.native_image(tauri::NativeImage::Add);
  }

  // create a submenu
  let my_sub_menu = Menu::new().add_item(disable_item);

  let my_app_menu = Menu::new()
    .add_native_item(MenuItem::Copy)
    .add_submenu(Submenu::new("Sub menu", my_sub_menu));

  let test_menu = Menu::new()
    .add_item(CustomMenuItem::new(
      "selected/disabled",
      "Selected and disabled",
    ))
    .add_native_item(MenuItem::Separator)
    .add_item(test_item);

  // add all our childs to the menu (order is how they'll appear)
  Menu::new()
    .add_submenu(Submenu::new("My app", my_app_menu))
    .add_submenu(Submenu::new("Other menu", test_menu))
}
