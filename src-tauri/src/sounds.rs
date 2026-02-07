use std::io::Cursor;
use std::sync::mpsc;
use std::thread;

pub const START_TONE: &[u8] = include_bytes!("../sounds/start.wav");
pub const STOP_TONE: &[u8] = include_bytes!("../sounds/stop.wav");

pub struct SoundPlayer {
    tx: mpsc::Sender<Vec<u8>>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            // OutputStream is !Send — must be created and held on this thread
            let (_stream, stream_handle) = match rodio::OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to open audio output stream: {}", e);
                    return;
                }
            };

            while let Ok(wav_bytes) = rx.recv() {
                let cursor = Cursor::new(wav_bytes);
                match stream_handle.play_once(cursor) {
                    Ok(sink) => sink.detach(),
                    Err(e) => eprintln!("Failed to play sound: {}", e),
                }
            }
        });

        Self { tx }
    }

    pub fn play(&self, wav_bytes: &[u8]) {
        let _ = self.tx.send(wav_bytes.to_vec());
    }
}
