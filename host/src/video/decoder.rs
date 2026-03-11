//! 视频帧解码器。
//!
//! 后台线程通过 ffmpeg-sidecar 驱动 FFmpeg 子进程，
//! 输出 RGBA 帧通过有界 channel 发送给主线程。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use ffmpeg_sidecar::command::FfmpegCommand;
use tracing::{debug, error};

use super::{VideoError, VideoFrame};

/// 帧缓冲上限（有界 channel 容量）。
///
/// 限制解码器最多领先渲染 2 帧，防止无界堆积导致内存膨胀。
/// 当 channel 满时解码线程自然阻塞等待消费。
const FRAME_BUFFER_CAPACITY: usize = 2;

/// 视频帧解码器
///
/// 在后台线程中运行 FFmpeg 解码，通过有界 channel 传递 RGBA 帧。
pub struct VideoDecoder {
    /// Option 包装以支持 stop() 时提前 drop receiver，
    /// 解除解码线程在 sync_channel::send() 上的阻塞。
    receiver: Option<mpsc::Receiver<VideoFrame>>,
    stop_flag: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    decode_thread: Option<JoinHandle<()>>,
}

impl VideoDecoder {
    /// 启动视频解码。
    ///
    /// 创建 FFmpeg 子进程和后台解码线程，立即返回。
    /// 帧通过 `next_frame()` 获取。
    pub fn start(video_path: &str) -> Result<Self, VideoError> {
        let (tx, rx) = mpsc::sync_channel(FRAME_BUFFER_CAPACITY);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));

        let stop = stop_flag.clone();
        let done = finished.clone();
        let path = video_path.to_string();

        let handle = thread::spawn(move || {
            let result = Self::decode_loop(&path, tx, &stop);
            if let Err(e) = result {
                error!(error = %e, "Video decode thread error");
            }
            done.store(true, Ordering::Release);
        });

        Ok(Self {
            receiver: Some(rx),
            stop_flag,
            finished,
            decode_thread: Some(handle),
        })
    }

    fn decode_loop(
        path: &str,
        tx: mpsc::SyncSender<VideoFrame>,
        stop: &AtomicBool,
    ) -> Result<(), VideoError> {
        let mut cmd = FfmpegCommand::new();
        // 手动指定输出格式：rawvideo + rgba。
        // 不使用 rawvideo() 快捷方法，因为它硬编码 rgb24 并在 pix_fmt 之后写 `-`（输出目标），
        // 导致追加的 -pix_fmt rgba 落在输出之后无效。
        // ffmpeg-sidecar 的 filter_frames() 会从 FFmpeg stderr 解析实际 pix_fmt，
        // 按 rgba=32bpp 正确计算帧大小。
        cmd.input(path)
            .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"]);

        #[cfg(windows)]
        cmd.create_no_window();

        let mut child = cmd
            .spawn()
            .map_err(|e| VideoError::ProcessError(format!("Failed to spawn FFmpeg: {e}")))?;

        let iter = child.iter().map_err(|e| {
            VideoError::ProcessError(format!("Failed to create frame iterator: {e}"))
        })?;

        debug!("Video decode thread started");

        for frame in iter.filter_frames() {
            if stop.load(Ordering::Relaxed) {
                debug!("Video decode stopped by request");
                break;
            }

            let video_frame = VideoFrame {
                width: frame.width,
                height: frame.height,
                data: frame.data,
                timestamp: frame.timestamp,
            };

            if tx.send(video_frame).is_err() {
                break; // receiver dropped
            }
        }

        debug!("Video decode thread finished");
        Ok(())
    }

    /// 获取下一个已解码帧（非阻塞）。
    ///
    /// 返回 None 表示当前无可用帧（可能仍在解码或已结束）。
    pub fn next_frame(&self) -> Option<VideoFrame> {
        self.receiver.as_ref()?.try_recv().ok()
    }

    /// 解码线程是否已结束。
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    /// 请求停止解码并等待线程退出。
    ///
    /// 先 drop receiver 解除解码线程在 `sync_channel::send()` 上的阻塞，
    /// 再 join 等待线程退出。
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
        // 必须先 drop receiver：解码线程可能阻塞在 sync_channel::send()，
        // drop receiver 使 send() 返回 Err，线程随即退出循环。
        self.receiver = None;
        if let Some(handle) = self.decode_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        self.stop();
    }
}
