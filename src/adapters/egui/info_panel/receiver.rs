//! 信息面板 EXIF 异步接收与轮询逻辑

use super::ExifData;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use tracing::error;

#[derive(Debug)]
pub(super) struct ExifLoadResult {
    pub(super) request_id: u64,
    pub(super) path: std::path::PathBuf,
    pub(super) exif_data: ExifData,
}

pub(super) enum ExifReceiveState {
    Loaded(Box<ExifLoadResult>),
    Pending,
    Disconnected,
}

pub(super) fn poll_exif_receiver(receiver: &Receiver<ExifLoadResult>) -> ExifReceiveState {
    match receiver.try_recv() {
        Ok(result) => ExifReceiveState::Loaded(Box::new(result)),
        Err(TryRecvError::Empty) => ExifReceiveState::Pending,
        Err(TryRecvError::Disconnected) => ExifReceiveState::Disconnected,
    }
}

pub(super) fn spawn_exif_loader(
    path: &Path,
    request_id: u64,
    reader: fn(&Path) -> ExifData,
) -> Receiver<ExifLoadResult> {
    let path = path.to_path_buf();
    let (sender, receiver) = channel::<ExifLoadResult>();

    thread::spawn(move || {
        let exif_data = reader(&path);
        if let Err(e) = sender.send(ExifLoadResult {
            request_id,
            path,
            exif_data,
        }) {
            error!("发送EXIF数据失败: {:?}", e);
        }
    });

    receiver
}
