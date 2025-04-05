use std::fs::{DirEntry, File};
use std::io::Cursor;
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
use hyper::body::Bytes;
use hyper::header;
use reqwest::header::HeaderMap;
use rodio::OutputStream;
use rodio::{Decoder, Sink};
use symphonia::core::audio::{AudioBufferRef, Channels, SampleBuffer, SignalSpec};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{
  MediaSource, MediaSourceStream, MediaSourceStreamOptions, ReadOnlySource,
};
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;

use lofty::{read_from_path, ParseOptions, TaggedFileExt};

use serde::{Deserialize, Serialize};
use std::ffi::OsString;

use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError;

use std::{string, thread};

use crossbeam::{
  atomic::AtomicCell,
  channel::{unbounded, Receiver, Sender},
  queue::ArrayQueue,
  sync::WaitGroup,
};

use std::time::{Duration, Instant};

use std::sync::{Arc, Mutex};

use crate::output;
use crate::state::{Play, PlayState};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
  pub id: String,
  pub title: String,
  pub artist: String,
  pub album: String,
  pub filename: String,
  pub img: String,
  pub duration: String,
}

impl Song {
  pub fn new() -> Song {
    Song {
      id: "".to_string(),
      title: "".to_string(),
      artist: "".to_string(),
      album: "".to_string(),
      filename: "".to_string(),
      img: "".to_string(),
      duration: "".to_string(),
    }
  }
}

#[tokio::main]
pub async fn streaming_audio(
  arg: &String,
) -> std::result::Result<std::string::String, reqwest::Error> {
  // let mut respons = client.get(arg).send().await.expect("url error");
  let response = reqwest::get(arg).await.unwrap().text().await;

  response
}

pub fn read_audio(arg: String) {
  let source = Box::new(ReadOnlySource::new(std::io::stdin())) as Box<dyn MediaSource>;

  // Create the media source stream using the boxed media source from above.
  let mss: MediaSourceStream = MediaSourceStream::new(source, Default::default());

  // Create a hint to help the format registry guess what format reader is appropriate. In this
  // example we'll leave it empty.
  let mut hint = Hint::new();
  hint.with_extension("aac");

  // Use the default options when reading and decoding.
  let format_opts: FormatOptions = Default::default();
  let metadata_opts: MetadataOptions = Default::default();
  let decoder_opts: DecoderOptions = Default::default();

  // Probe the media source stream for a format.
  let probed = symphonia::default::get_probe()
    .format(&hint, mss, &format_opts, &metadata_opts)
    .unwrap();

  // Get the format reader yielded by the probe operation.
  let mut format = probed.format;
  let mut probedmetadata = probed.metadata;

  let mut metadata = probedmetadata.get().unwrap();
  // Consume any new metadata that has been read since the last packet.
  while !metadata.is_latest() {
    // Pop the old head of the metadata queue.
    metadata.pop();
  }
  let metadata_revision = metadata.current().unwrap();
  let tag_iter = metadata_revision.tags().iter();
  let image_iter = metadata_revision.visuals().iter();

  for i in tag_iter {
    println!("{i}");
    if i.key.contains("PRIV") {
      // let mut v: Vec<u8> = i
      //   .value
      //   .to_string()
      //   .split("\\")
      //   .filter(|s| s.len() != 0)
      //   .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
      //   .collect(); //.map(|s| s.parse().unwrap()).collect();
      // println!("{:?}", String::from_utf8_lossy(&v));
      println!("{}", OsString::from(i.value.to_string()).to_string_lossy());
    }
  }

  for i in image_iter {
    // println!("{:?}", i);
  }

  // Get the default track.
  let track = format.default_track().unwrap();

  // Create a decoder for the track.
  let mut decoder = symphonia::default::get_codecs()
    .make(&track.codec_params, &decoder_opts)
    .unwrap();

  // Store the track identifier, we'll use it to filter packets.
  let track_id = track.id;

  // let mut sample_count = 0;
  // let mut sample_buf = None;

  // loop {
  //   // Get the next packet from the format reader.
  //   let packet = format.next_packet().unwrap();

  //   // If the packet does not belong to the selected track, skip it.
  //   if packet.track_id() != track_id {
  //     continue;
  //   }

  //   // Decode the packet into audio samples, ignoring any decode errors.
  //   match decoder.decode(&packet) {
  //     Ok(audio_buf) => {
  //       // The decoded audio samples may now be accessed via the audio buffer if per-channel
  //       // slices of samples in their native decoded format is desired. Use-cases where
  //       // the samples need to be accessed in an interleaved order or converted into
  //       // another sample format, or a byte buffer is required, are covered by copying the
  //       // audio buffer into a sample buffer or raw sample buffer, respectively. In the
  //       // example below, we will copy the audio buffer into a sample buffer in an
  //       // interleaved order while also converting to a f32 sample format.

  //       // If this is the *first* decoded packet, create a sample buffer matching the
  //       // decoded audio buffer format.
  //       if sample_buf.is_none() {
  //         // Get the audio buffer specification.
  //         let spec = *audio_buf.spec();

  //         // Get the capacity of the decoded buffer. Note: This is capacity, not length!
  //         let duration = audio_buf.capacity() as u64;

  //         // Create the f32 sample buffer.
  //         sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
  //       }

  //       // Copy the decoded audio buffer into the sample buffer in an interleaved format.
  //       if let Some(buf) = &mut sample_buf {
  //         buf.copy_interleaved_ref(audio_buf);

  //         // The samples may now be access via the `samples()` function.
  //         sample_count += buf.samples().len();
  //         print!("\rDecoded {} samples", sample_count);
  //       }
  //     }
  //     Err(Error::DecodeError(_)) => (),
  //     Err(_) => break,
  //   }
  // }
}

fn run_producer_chan(s: Sender<u32>, num: u32, a: Arc<Mutex<bool>>) -> thread::JoinHandle<()> {
  thread::spawn(move || {
    let c = a.lock().unwrap();
    println!("Hello from producer thread {} - pushing...!", num);
    for i in 0..1000 {
      // println!("{}", c);
      if i == 567 && *c {
        break;
      }

      s.send(num).expect("send failed");
    }
  })
}

fn run_consumer_chan(r: Receiver<u32>, num: u32) -> thread::JoinHandle<()> {
  thread::spawn(move || {
    let mut i = 0;
    println!("Hello from producer thread {} - popping!", num);
    loop {
      if let Err(_) = r.recv() {
        println!(
          "last sender dropped - stopping consumer thread, messages received: {}",
          i
        );
        break;
      }
      i += 1;
    }
  })
}

pub fn read_audio2(state: Arc<std::sync::Mutex<Play>>, argument: String, count: u32) {
  let path = argument;

  // Create a media source. Note that the MediaSource trait is automatically implemented for File,
  // among other types.
  let file = Box::new(File::open(path).unwrap());

  // Create the media source stream using the boxed media source from above.
  let mss: MediaSourceStream = MediaSourceStream::new(file, Default::default());

  // Create a hint to help the format registry guess what format reader is appropriate. In this
  // example we'll leave it empty.
  let mut hint = Hint::new();
  hint.with_extension("mp3");

  // Use the default options when reading and decoding.
  let format_opts: FormatOptions = Default::default();
  let metadata_opts: MetadataOptions = Default::default();
  let decoder_opts: DecoderOptions = Default::default();

  // Probe the media source stream for a format.
  let probed = symphonia::default::get_probe()
    .format(&hint, mss, &format_opts, &metadata_opts)
    .unwrap();

  // Get the format reader yielded by the probe operation.
  let mut format = probed.format;
  let mut probedmetadata = probed.metadata;

  let mut metadata = probedmetadata.get().unwrap();
  // Consume any new metadata that has been read since the last packet.
  while !metadata.is_latest() {
    // Pop the old head of the metadata queue.
    metadata.pop();
  }
  let metadata_revision = metadata.current().unwrap();
  let tag_iter = metadata_revision.tags().iter();
  let image_iter = metadata_revision.visuals().iter();

  for i in tag_iter {
    println!("{i}");
    if i.key.contains("PRIV") {
      // let mut v: Vec<u8> = i
      //   .value
      //   .to_string()
      //   .split("\\")
      //   .filter(|s| s.len() != 0)
      //   .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
      //   .collect(); //.map(|s| s.parse().unwrap()).collect();
      // println!("{:?}", String::from_utf8_lossy(&v));
      println!("{}", OsString::from(i.value.to_string()).to_string_lossy());
    }
  }

  // Get the default track.
  let track = format.default_track().unwrap();

  // Create a decoder for the track.
  let mut decoder = symphonia::default::get_codecs()
    .make(&track.codec_params, &decoder_opts)
    .unwrap();

  // Store the track identifier, we'll use it to filter packets.
  let track_id = track.id;

  let mut sample_count = 0;
  let mut sample_buf = None;

  // The audio output device.
  let mut audio_output = None;

  while let Ok(packet) = format.next_packet() {
    // If the packet does not belong to the selected track, skip it.
    if packet.track_id() != track_id {
      continue;
    }

    let mut c = state.lock().unwrap();
    if (*c).is_same_count(count) == false {
      break;
    }

    // Decode the packet into audio samples, ignoring any decode errors.
    match decoder.decode(&packet) {
      Ok(audio_buf) => {
        // The decoded audio samples may now be accessed via the audio buffer if per-channel
        // slices of samples in their native decoded format is desired. Use-cases where
        // the samples need to be accessed in an interleaved order or converted into
        // another sample format, or a byte buffer is required, are covered by copying the
        // audio buffer into a sample buffer or raw sample buffer, respectively. In the
        // example below, we will copy the audio buffer into a sample buffer in an
        // interleaved order while also converting to a f32 sample format.

        // If this is the *first* decoded packet, create a sample buffer matching the
        // decoded audio buffer format.
        // Get the audio buffer specification.
        let spec = *audio_buf.spec();

        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
        let duration = audio_buf.capacity() as u64;

        if sample_buf.is_none() {
          // Create the f32 sample buffer.
          sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }

        // Copy the decoded audio buffer into the sample buffer in an interleaved format.
        if let Some(buf) = &mut sample_buf {}

        // buf.copy_interleaved_ref(audio_buf);

        // // The samples may now be access via the `samples()` function.
        // sample_count += buf.samples().len();
        // print!("\rDecoded {} samples", sample_count);

        // If the audio output is not open, try to open it.
        if audio_output.is_none() {
          // Try to open the audio output.
          audio_output.replace(output::try_open(spec, duration).unwrap());
        } else {
          // TODO: Check the audio spec. and duration hasn't changed.
        }

        if let Some(ref mut audio_output) = audio_output {
          audio_output.write(audio_buf).unwrap()
          // send(audio_buf);
        }
      }
      Err(Error::DecodeError(_)) => (),
      Err(_) => break,
    }
  }
}

pub fn read_buffer_path(path: String) {
  // Create the media source stream using the boxed media source from above.
  let file = Box::new(File::open(path).unwrap()) as Box<dyn MediaSource>;
  let mss: MediaSourceStream = MediaSourceStream::new(file, Default::default());

  // 포맷을 감지합니다.
  let hint = Hint::new();
  let format_opts = Default::default();
  let metadata_opts = MetadataOptions::default();
  // Probe the media source stream for a format.
  let probed =
    match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
      Ok(probed) => probed,
      Err(Error::IoError(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
        eprintln!("Unexpected EOF");
        return;
      }
      Err(err) => {
        eprintln!("Error probing format: {:?}", err);
        return;
      }
    };

  let mut format = probed.format;

  // 디코더를 생성합니다.
  let track = format.default_track().unwrap();
  let mut decoder = symphonia::default::get_codecs()
    .make(&track.codec_params, &Default::default())
    .unwrap();

  // Store the track identifier, we'll use it to filter packets.
  let track_id = track.id;

  let mut sample_buf = None;

  // The audio output device.
  let mut audio_output = None;

  while let Ok(packet) = format.next_packet() {
    // If the packet does not belong to the selected track, skip it.
    if packet.track_id() != track_id {
      continue;
    }

    // let c = state.lock().unwrap();
    // if (*c).is_same_count(count) == false {
    //   break;
    // }

    // Decode the packet into audio samples, ignoring any decode errors.
    match decoder.decode(&packet) {
      Ok(audio_buf) => {
        // The decoded audio samples may now be accessed via the audio buffer if per-channel
        // slices of samples in their native decoded format is desired. Use-cases where
        // the samples need to be accessed in an interleaved order or converted into
        // another sample format, or a byte buffer is required, are covered by copying the
        // audio buffer into a sample buffer or raw sample buffer, respectively. In the
        // example below, we will copy the audio buffer into a sample buffer in an
        // interleaved order while also converting to a f32 sample format.

        // If this is the *first* decoded packet, create a sample buffer matching the
        // decoded audio buffer format.
        // Get the audio buffer specification.
        let spec = *audio_buf.spec();

        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
        let duration = audio_buf.capacity() as u64;

        if sample_buf.is_none() {
          // Create the f32 sample buffer.
          sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }

        // Copy the decoded audio buffer into the sample buffer in an interleaved format.
        if let Some(buf) = &mut sample_buf {}

        // buf.copy_interleaved_ref(audio_buf);

        // // The samples may now be access via the `samples()` function.
        // sample_count += buf.samples().len();
        // print!("\rDecoded {} samples", sample_count);

        // If the audio output is not open, try to open it.
        if audio_output.is_none() {
          // Try to open the audio output.
          audio_output.replace(output::try_open(spec, duration).unwrap());
        } else {
          // TODO: Check the audio spec. and duration hasn't changed.
        }

        if let Some(ref mut audio_output) = audio_output {
          audio_output.write(audio_buf).unwrap()
          // send(audio_buf);
        }
      }
      Err(Error::DecodeError(_)) => (),
      Err(_) => break,
    }
  }
}

pub fn read_buffer(header: HeaderMap, bytes: Bytes) {
  println!("{:?}", header);

  // // Cursor를 사용하여 버퍼를 읽을 수 있는 스트림으로 변환합니다.
  // let cursor = Cursor::new(bytes);

  // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
  // let sink = Sink::try_new(&stream_handle).unwrap();

  // // 디코더를 사용하여 오디오 데이터를 디코딩합니다.
  // match Decoder::new(cursor) {
  //   Ok(source) => {
  //     // 싱크에 오디오 소스를 추가하고 재생을 시작합니다.
  //     sink.append(source);
  //     sink.sleep_until_end();
  //   }
  //   Err(e) => {
  //     eprintln!("Failed to decode audio: {:?}", e);
  //   }
  // }

  // // let channels = Channels::FRONT_LEFT | Channels::FRONT_CENTRE | Channels::FRONT_RIGHT;
  // // let spec = SignalSpec::new(44100, channels);

  // // // The audio output device.
  // // let mut audio_output = None;
  // // audio_output.replace(output::try_open(spec, 1152).unwrap());
  // // if let Some(ref mut audio_output) = audio_output {
  // //   let audio_buf = AudioBufferRef::new(buffer, spec);
  // //   audio_output.write(&audio_buf).unwrap();
  // //   // send(audio_buf);
  // // }

  // println!("{:?}", header);
  // // println!("{:?}", bytes);

  // Cursor를 사용하여 버퍼를 읽을 수 있는 스트림으로 변환합니다.
  let cursor = Cursor::new(bytes);
  // Create the media source stream using the boxed media source from above.
  let mss: MediaSourceStream = MediaSourceStream::new(Box::new(cursor), Default::default());

  // 포맷을 감지합니다.
  let hint = Hint::new();
  let format_opts = Default::default();
  let metadata_opts = MetadataOptions::default();
  // Probe the media source stream for a format.
  let probed =
    match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
      Ok(probed) => probed,
      Err(Error::IoError(err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
        eprintln!("Unexpected EOF");
        return;
      }
      Err(err) => {
        eprintln!("Error probing format: {:?}", err);
        return;
      }
    };

  let mut format = probed.format;

  // 디코더를 생성합니다.
  let track = format.default_track().unwrap();
  let mut decoder = symphonia::default::get_codecs()
    .make(&track.codec_params, &Default::default())
    .unwrap();

  // Store the track identifier, we'll use it to filter packets.
  let track_id = track.id;

  let mut sample_buf = None;

  // The audio output device.
  let mut audio_output = None;

  while let Ok(packet) = format.next_packet() {
    // If the packet does not belong to the selected track, skip it.
    if packet.track_id() != track_id {
      continue;
    }

    // let c = state.lock().unwrap();
    // if (*c).is_same_count(count) == false {
    //   break;
    // }

    // Decode the packet into audio samples, ignoring any decode errors.
    match decoder.decode(&packet) {
      Ok(audio_buf) => {
        // The decoded audio samples may now be accessed via the audio buffer if per-channel
        // slices of samples in their native decoded format is desired. Use-cases where
        // the samples need to be accessed in an interleaved order or converted into
        // another sample format, or a byte buffer is required, are covered by copying the
        // audio buffer into a sample buffer or raw sample buffer, respectively. In the
        // example below, we will copy the audio buffer into a sample buffer in an
        // interleaved order while also converting to a f32 sample format.

        // If this is the *first* decoded packet, create a sample buffer matching the
        // decoded audio buffer format.
        // Get the audio buffer specification.
        let spec = *audio_buf.spec();

        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
        let duration = audio_buf.capacity() as u64;

        if sample_buf.is_none() {
          // Create the f32 sample buffer.
          sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
        }

        // Copy the decoded audio buffer into the sample buffer in an interleaved format.
        if let Some(buf) = &mut sample_buf {}

        // buf.copy_interleaved_ref(audio_buf);

        // // The samples may now be access via the `samples()` function.
        // sample_count += buf.samples().len();
        // print!("\rDecoded {} samples", sample_count);

        // If the audio output is not open, try to open it.
        if audio_output.is_none() {
          // Try to open the audio output.
          audio_output.replace(output::try_open(spec, duration).unwrap());
        } else {
          // TODO: Check the audio spec. and duration hasn't changed.
        }

        if let Some(ref mut audio_output) = audio_output {
          audio_output.write(audio_buf).unwrap()
          // send(audio_buf);
        }
      }
      Err(Error::DecodeError(_)) => (),
      Err(_) => break,
    }
  }
}

pub fn play_base64_audio(base64_str: &str) {
  match STANDARD.decode(base64_str) {
    Ok(decoded_bytes) => {
      // Cursor를 사용하여 버퍼를 읽을 수 있는 스트림으로 변환합니다.
      let cursor = Cursor::new(decoded_bytes);

      // 오디오 출력 스트림과 싱크를 생성합니다.
      let (_stream, stream_handle) = OutputStream::try_default().unwrap();
      let sink = Sink::try_new(&stream_handle).unwrap();

      // 디코더를 사용하여 오디오 데이터를 디코딩합니다.
      match Decoder::new(cursor) {
        Ok(source) => {
          // 싱크에 오디오 소스를 추가하고 재생을 시작합니다.
          sink.append(source);
          sink.sleep_until_end();
        }
        Err(e) => {
          eprintln!("Failed to decode audio: {:?}", e);
        }
      }
    }
    Err(e) => {
      eprintln!("Failed to decode base64 string: {:?}", e);
    }
  }
}

// pub async fn fetch_audio_stream(
//   url: &str,
// ) -> Result<MediaSourceStream<Box<dyn std::io::Read + Send>>, reqwest::Error> {
//   // let client = Client::new();
//   // let response = client.get(url).send().await?;
//   // let bytes = response.bytes().await?;

//   // let cursor = Cursor::new(bytes);
//   // let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

//   // Ok(mss)
//   Ok(MediaSourceStream::new())
// }

// pub async fn play_audio_from_url(url: &str) {
//   match fetch_audio_stream(url).await {
//     Ok(mss) => {
//       let hint = Hint::new();
//       let format_opts = FormatOptions::default();
//       let metadata_opts = MetadataOptions::default();
//       let decoder_opts = DecoderOptions::default();

//       let probed =
//         match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
//           Ok(probed) => probed,
//           Err(err) => {
//             eprintln!("Error probing format: {:?}", err);
//             return;
//           }
//         };

//       let mut format = probed.format;

//       let track = match format.default_track() {
//         Some(track) => track,
//         None => {
//           eprintln!("No default track found");
//           return;
//         }
//       };

//       let mut decoder =
//         match symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts) {
//           Ok(decoder) => decoder,
//           Err(err) => {
//             eprintln!("Error creating decoder: {:?}", err);
//             return;
//           }
//         };

//       // 오디오 재생 로직 추가
//       // 예: rodio를 사용하여 재생
//       // let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
//       // let source = rodio::Decoder::new(mss).unwrap();
//       // stream_handle.play_raw(source.convert_samples()).unwrap();
//       // std::thread::sleep(std::time::Duration::from_secs(10));
//     }
//     Err(err) => {
//       eprintln!("Error fetching audio stream: {:?}", err);
//     }
//   }
// }

pub fn read_files() -> Vec<Song> {
  // const audioDownloadPath = path.join(
  //   app.getPath('documents'),
  //   '멜론 보관함',
  //   '받은 파일함'
  // );

  let mut v: Vec<Song> = Vec::new();

  let paths = std::fs::read_dir("./assets").unwrap();

  for path in paths {
    // println!("{:?}", path.unwrap().file_name());
    // println!("Name: {}", path.unwrap().path().display());

    if let Ok(f) = path {
      let file = Box::new(File::open(f.path()).unwrap());

      let mut title: String = String::new();
      let mut artist: String = String::new();
      let mut album: String = String::new();

      // Create the media source stream using the boxed media source from above.
      let mss: MediaSourceStream = MediaSourceStream::new(file, Default::default());

      // Create a hint to help the format registry guess what format reader is appropriate. In this
      // example we'll leave it empty.
      let mut hint = Hint::new();
      hint.with_extension("mp3");

      // Use the default options when reading and decoding.
      let format_opts: FormatOptions = Default::default();
      let metadata_opts: MetadataOptions = Default::default();
      let decoder_opts: DecoderOptions = Default::default();

      // Probe the media source stream for a format.
      let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .unwrap();

      // Get the format reader yielded by the probe operation.
      let mut format = probed.format;
      let mut probedmetadata = probed.metadata;

      let mut metadata = probedmetadata.get().unwrap();
      // Consume any new metadata that has been read since the last packet.
      while !metadata.is_latest() {
        // Pop the old head of the metadata queue.
        metadata.pop();
      }
      let metadata_revision = metadata.current().unwrap();
      let tag_iter = metadata_revision.tags().iter();
      let image_iter = metadata_revision.visuals().iter();

      for i in tag_iter {
        // if i.key.contains("PRIV") {
        //   let v: Vec<u8> = i
        //     .value
        //     .to_string()
        //     .split("\\")
        //     .filter(|s| s.len() != 0)
        //     .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
        //     .collect();
        //   println!("{:?}", String::from_utf8_lossy(&v));
        // }
        // println!("{i} {:?}", f.path().into_os_string());
        // i.std_key

        match i.std_key {
          Some(key) => match key {
            StandardTagKey::Artist => artist = i.value.to_string(),
            StandardTagKey::Album => album = i.value.to_string(),
            StandardTagKey::TrackTitle => title = i.value.to_string(),
            _ => (),
          },
          None => {}
        }
      }

      v.push(Song {
        id: "0".into(),
        title: title,
        artist: artist,
        album: album,
        filename: f.path().into_os_string().into_string().unwrap(),
        img: String::from("abc"),
        duration: String::from("0"),
      });
    }
  }

  v
}
