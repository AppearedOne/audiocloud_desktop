use anyhow::Result;
use rodio::{source::Source, Decoder, OutputStream, OutputStreamHandle, Sink};

pub async fn play_audio() -> Result<(), anyhow::Error> {
    Ok(())
}

pub struct Handlers {
    pub stream: OutputStream,
    pub stream_handle: OutputStreamHandle,
    pub sink: Sink,
}
pub async fn init_audio_unwrap() -> Handlers {
    init_audio().expect("Couldnt init audio devices")
}
pub fn init_audio() -> Result<Handlers, anyhow::Error> {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    Ok(Handlers {
        stream,
        stream_handle,
        sink,
    })
}
