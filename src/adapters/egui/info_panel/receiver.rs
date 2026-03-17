use super::ExifData;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use tracing::error;

pub(super) enum ExifReceiveState {
    Loaded(Box<ExifData>),
    Pending,
    Disconnected,
}

pub(super) fn poll_exif_receiver(receiver: &Receiver<ExifData>) -> ExifReceiveState {
    match receiver.try_recv() {
        Ok(exif_data) => ExifReceiveState::Loaded(Box::new(exif_data)),
        Err(TryRecvError::Empty) => ExifReceiveState::Pending,
        Err(TryRecvError::Disconnected) => ExifReceiveState::Disconnected,
    }
}

pub(super) fn spawn_exif_loader(path: &Path, reader: fn(&Path) -> ExifData) -> Receiver<ExifData> {
    let path = path.to_path_buf();
    let (sender, receiver) = channel::<ExifData>();

    thread::spawn(move || {
        let exif_data = reader(&path);
        if let Err(e) = sender.send(exif_data) {
            error!("发送EXIF数据失败: {:?}", e);
        }
    });

    receiver
}
