use crate::{
  cmd,
  other::{self, Song},
  GlobalApp,
};
use bytes::Bytes;
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::{Cursor, Write};
use std::{
  ffi::CString,
  sync::{Arc, Mutex},
};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::StoreCollection;

use base64::{engine::general_purpose::STANDARD, Engine};
use hls_m3u8::MediaPlaylist;
use reqwest::Client;
use std::collections::HashMap as HeaderMap;
use std::thread;

const MAX: u32 = 3;

#[derive(Debug)]
pub enum State {
  Play,
  Pause,
  Stop,
}

pub struct PlayState(pub Arc<Mutex<Play>>);

pub struct AudioPlayer {
  sink: Arc<Mutex<Sink>>,
}

impl AudioPlayer {
  pub fn new() -> Option<Self> {
    let (_stream, stream_handle) = match OutputStream::try_default() {
      Ok(s) => s,
      Err(e) => {
        log::error!("Error! Failed to create output stream. {}", e);
        return None;
      }
    };

    let sink = match Sink::try_new(&stream_handle) {
      Ok(s) => s,
      Err(e) => {
        log::error!("Error! Could not create sink. {}", e);
        return None;
      }
    };

    Some(Self {
      sink: Arc::new(Mutex::new(sink)),
    })
  }

  pub fn init(&mut self, buffer: Vec<u8>) -> Self {
    // Cursor를 사용하여 버퍼를 읽을 수 있는 스트림으로 변환합니다.
    let cursor = Cursor::new(buffer);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // 디코더를 사용하여 오디오 데이터를 디코딩합니다.
    match Decoder::new(cursor) {
      Ok(source) => {
        // 싱크에 오디오 소스를 추가하고 재생을 시작합니다.
        sink.append(source);
        sink.set_volume(0.25);
      }
      Err(e) => {
        eprintln!("Failed to decode audio: {:?}", e);
      }
    }

    let sink = Arc::new(Mutex::new(sink));
    let sink_clone = Arc::clone(&sink);

    thread::spawn(move || {
      let thread_sink = sink_clone.lock().unwrap();
      thread_sink.sleep_until_end();
      let g = GlobalApp::global();
      let _ = g.app.emit_all("next", "");
    });

    Self { sink }
  }

  pub fn pause(&self) {
    let sink = self.sink.lock().unwrap();
    sink.pause();
  }

  pub fn play(&self) {
    let sink = self.sink.lock().unwrap();
    sink.play();
  }

  pub fn stop(&self) {
    let sink = self.sink.lock().unwrap();
    sink.stop();
  }
}

impl Drop for AudioPlayer {
  fn drop(&mut self) {
    self.stop();
  }
}

pub struct Play {
  pub count: u32,
  pub state: State,
  pub app_handler: AppHandle,
  pub list: Vec<Song>,
  pub index: i32,
  pub port: u16,
  pub token: String,
  pub session_id: String,
  pub hls: bool,
  pub player: Option<AudioPlayer>,
}

impl Play {
  pub fn new(app: AppHandle) -> Play {
    Play {
      count: 0,
      state: State::Stop,
      app_handler: app,
      list: vec![],
      index: 0,
      port: 0,
      token: "".to_string(),
      session_id: "".to_string(),
      hls: true,
      player: AudioPlayer::new(),
    }
  }

  pub fn increase(&mut self) {
    let c = (*self).count + 1;
    (*self).count = c % MAX
  }

  pub fn get_count(&self) -> u32 {
    (*self).count
  }

  pub fn is_same_count(&self, count: u32) -> bool {
    (*self).count == count
  }

  pub fn set_port(&mut self, port: u16) {
    (*self).port = port;
  }

  pub fn get_port(&self) -> u16 {
    (*self).port
  }

  pub fn set_token(&mut self, t: String) {
    (*self).token = t;
  }

  pub fn set_session_id(&mut self, s: String) {
    (*self).session_id = s;
  }

  pub fn get_token(&self) -> String {
    (*self).token.clone()
  }

  pub fn get_session_id(&self) -> String {
    (*self).session_id.clone()
  }

  pub fn toggle_hls(&mut self) {
    let hls = (*self).hls;
    (*self).hls = !hls;
  }

  pub fn get_hls(&self) -> bool {
    (*self).hls
  }

  // pub fn is_playing(&self) -> bool {
  //   match self.state {
  //     State::Play => true,
  //     _ => false,
  //   }
  // }

  // pub fn is_stop(&self) -> bool {
  //   match self.state {
  //     State::Stop => true,
  //     _ => false,
  //   }
  // }

  pub fn state_play(&mut self) {
    (*self).state = State::Play;
    if let Some(player) = &self.player {
      player.play();
    }
  }

  pub fn state_stop(&mut self) {
    (*self).state = State::Stop;
    if let Some(player) = &self.player {
      player.stop();
    }
  }

  pub fn state_pause(&mut self) {
    (*self).state = State::Pause;
    if let Some(player) = &self.player {
      player.pause();
    }
  }

  pub fn get_title(&self) -> Vec<String> {
    let song = self.list.get(self.index as usize).cloned();
    let title = match song {
      Some(s) => s.title.to_lowercase(),
      _ => "".to_string(),
    };

    let mut answer: Vec<String> = vec![];
    answer.push(title.clone());
    answer.push(title.replace(" ", ""));
    match title.split_once('(') {
      Some((key, value)) => {
        answer.push(key.to_string());
        answer.push(key.to_string().replace(" ", ""));
      }
      None => (),
    };

    answer
  }

  pub fn play_buffer(&mut self, buffer: Vec<u8>) {
    match self.player {
      Some(ref mut player) => {
        player.init(buffer);
      }
      None => (),
    }
  }

  pub fn play(&mut self) {
    let song = self.list.get(self.index as usize).cloned();
    if let Some(s) = song {
      let res = cmd::get_path(
        s.id.clone(),
        self.get_hls(),
        self.get_token(),
        self.get_session_id(),
      );
      if let Ok(r) = res {
        let a = r.get("response").unwrap();
        println!("{:#?}", r);
        let b = match a.get("stInfo") {
          Some(st_info) => st_info,
          None => {
            // (*self).next();
            return;
          }
        };
        let c = b.get("path").unwrap().to_string().replace("\"", "");
        println!("{:#?}", c);
        let g = GlobalApp::global();
        if c.contains("m3u8") {
          let _ = g.app.emit_all("hls", c);
        } else if c.contains("mcache") {
          println!("{:#?}", b);
          let key = b.get("c").unwrap().to_string().replace("\"", "");
          let size = b.get("fileSize").unwrap().to_string().replace("\"", "");
          let d = c.replace("mcache", "https");
          let ts_res = cmd::get_mcache(&d, u32::into(size.to_string().parse().unwrap()));
          // let cache = mcache::mcache::read_audio_buffer(ts_res, key, 0);
          // println!("{:#?}", ts_res);
          match ts_res {
            Ok(response) => {
              let (header, bytes) = response;

              unsafe {
                let key_len = key.len() as i32;
                // utf8 문자열을 c style 에 문자열로 바꿈
                let c_key = CString::new(key).unwrap();
                let c_key_ptr = c_key.as_ptr();
                let ctx = initialize(c_key_ptr as *mut i8, key_len, 0);
                println!("{:#?} {:#?} {:#?}", c_key, c_key_ptr, ctx);

                // 버퍼 메모리 할당
                let mut buffer: Vec<u8> = vec![0; bytes.len()];

                let buffer_size: libc::c_uint = 0;

                // bytes 값을 decrypt 하기 전 파일에 기록
                // let mut file = File::create("tmp.txt").unwrap();
                // writeln!(file, "Before decrypt: {:?}", bytes).unwrap();

                decrypt(
                  ctx,
                  bytes.as_ptr() as *mut i8,
                  bytes.len() as libc::c_uint as i32,
                  buffer.as_mut_ptr() as *mut i8,
                  buffer_size as *mut libc::c_uint as *mut i32,
                );

                // 오브젝트 리소스 해제 함수
                release(ctx);

                let chunk_size = 1024 * 1024; // 1MB 청크
                let _ = g.app.emit_all("chunk_start", "");
                for chunk in buffer.chunks(chunk_size) {
                  // let base64_audio = STANDARD.encode(chunk);
                  // other::play_base64_audio(&base64_audio);
                  let _ = g.app.emit_all("decrypted_audio_chunk", chunk.to_vec());
                }
                let _ = g.app.emit_all("chunk_end", "");
                // if buffer.len() < chunk_size {
                //   let base64_audio = STANDARD.encode(&buffer);
                //   let _ = g.app.emit_all("decrypted_audio", base64_audio);
                // } else {
                //   // self.play_buffer(buffer);
                //   let _ = g.app.emit_all("chunk_start", "");
                //   for chunk in buffer.chunks(chunk_size) {
                //     // let base64_audio = STANDARD.encode(chunk);
                //     // other::play_base64_audio(&base64_audio);
                //     let _ = g.app.emit_all("decrypted_audio_chunk", chunk.to_vec());
                //   }
                //   let _ = g.app.emit_all("chunk_end", "");
                // }

                // let base64_audio = STANDARD.encode(&buffer);
                // let _ = g.app.emit_all("decrypted_audio", base64_audio);
                // let bytes = Bytes::from(base64_audio);
                // writeln!(file, "After decrypt: {:?}", bytes).unwrap();
                // other::read_buffer(header, bytes);
              }
            }
            Err(_) => (),
          }
        }
        let _ = g.app.emit_all("song", s);
        let _ = g.app.emit_all("state", true);
      }
    }
  }

  //   pub fn response_write() {
  //     let mut headers = HeaderMap::new();
  //     headers.insert("Cache-control", HeaderValue::from_static("no-cache"));
  //     headers.insert(RANGE, HeaderValue::from_str(&size).unwrap());

  //   protected _responseWrite(res: Response, resInfo: { start: number; chunkSize: number; fileSize: number; contentType: string; resBody: Buffer }): void {
  //     const chunkBytes = resInfo.start + resInfo.chunkSize;
  //     const resHeaders = {
  //         Pragma: 'no-cache',
  //         'Cache-control': 'no-cache',
  //         'Content-Range': 'bytes ' + resInfo.start + '-' + (chunkBytes - 1) + '/' + resInfo.fileSize,
  //         'Accept-Ranges': 'bytes',
  //         'Content-Length': resInfo.chunkSize,
  //         'Content-Type': resInfo.contentType
  //     };

  //     logger.silly('responseWrite: start ', resInfo.start + '-' + (chunkBytes - 1) + '/' + resInfo.fileSize);

  //     res.writeHead(206, resHeaders);
  //     res.end(resInfo.resBody, 'binary');

  //     logger.silly('responseWrite: end ', resInfo.start + '-' + (chunkBytes - 1) + '/' + resInfo.fileSize);
  // }
  //   }

  pub fn next(&mut self) {
    let mut i = (*self).index;
    i = (i + 1) % (*self).list.len() as i32;
    (*self).index = i;
    self.play();
  }

  pub fn prev(&mut self) {
    let i = (*self).index - 1;
    (*self).index = if i >= 0 {
      i
    } else {
      (*self).list.len() as i32 - 1
    };
    self.play();
  }

  pub fn set_list(&mut self, v: Vec<Song>) {
    (*self).list = v;
    (*self).index = 0
  }

  pub fn play_or_pause(&mut self) {
    match self.state {
      State::Play => self.state_pause(),
      State::Pause => self.state_play(),
      State::Stop => self.state_play(),
    }
  }
}
