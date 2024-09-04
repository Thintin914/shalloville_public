use image::imageops::FilterType;
use image::RgbaImage;
use livekit::options::TrackPublishOptions;
use flume::bounded;
use livekit::prelude::*;
use livekit::webrtc::video_source::RtcVideoSource;
use livekit::webrtc::video_source::VideoResolution;
use livekit::webrtc::{
    native::yuv_helper,
    video_frame::{I420Buffer, VideoFrame, VideoRotation},
    video_source::native::NativeVideoSource,
};
use parking_lot::Mutex;
use std::sync::Arc;

// The logo must not be bigger than the framebuffer
const PIXEL_SIZE: usize = 4;
const MOVE_SPEED: i32 = 16;
const FB_WIDTH: usize = 1920;
const FB_HEIGHT: usize = 1080;
const COMPRESSED_WIDTH: usize = 480;
const COMPRESSED_HEIGHT: usize = 270;

#[derive(Clone)]
struct FrameData {
    image: RgbaImage,
    framebuffer: Arc<Mutex<Vec<u8>>>,
    video_frame: Arc<Mutex<VideoFrame<I420Buffer>>>,
    pos: (i32, i32),
    direction: (i32, i32),
}

struct TrackHandle {
    close_sender: flume::Sender<bool>,
    track: LocalVideoTrack
}

pub struct DeviceVideoTrack {
    rtc_source: NativeVideoSource,
    room: Arc<Mutex<Option<Room>>>,
    handle: Option<TrackHandle>,
}

impl DeviceVideoTrack {
    pub fn new(room: Arc<Mutex<Option<Room>>>) -> DeviceVideoTrack {
        DeviceVideoTrack {
            rtc_source: NativeVideoSource::new(VideoResolution {
                width: FB_WIDTH as u32,
                height: FB_HEIGHT as u32
            }),
            room,
            handle: None,
        }
    }

    pub async fn publish(&mut self, track_name: &str, image_receiver: flume::Receiver<RgbaImage>) {
        let rtc_source = self.rtc_source.clone();
        let room = self.room.clone();
        let (close_sender, close_receiver) = bounded::<bool>(1);

        let cloned_track_name = track_name.to_string();

        self.unpublish().await;

        let track = LocalVideoTrack::create_video_track(
            &cloned_track_name,
            RtcVideoSource::Native(rtc_source.clone()),
        );
                    
        if let Some(room) = room.lock().as_ref() {
            let _ = room.local_participant()
            .publish_track(
                LocalTrack::Video(track.clone()),
                TrackPublishOptions {
                    source: TrackSource::Camera,
                    ..Default::default()
                },
            )
            .await;
        }

        self.handle = Some(TrackHandle {
            close_sender,
            track,
        });
        self.track_task(close_receiver, self.rtc_source.clone(), image_receiver);
    }

    pub async fn unpublish(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.close_sender.send(true);
            
            if let Some(room) = self.room.lock().as_ref() {
                let _ = room
                .local_participant()
                .unpublish_track(&handle.track.sid())
                .await;
            }
        }
    }

    fn track_task(&self, close_rx: flume::Receiver<bool>, rtc_source: NativeVideoSource, image_receiver: flume::Receiver<RgbaImage>) {

        std::thread::spawn(move || {
            async_std::task::block_on(async {        
                let mut data = FrameData {
                    image: RgbaImage::new(0, 0),
                    framebuffer: Arc::new(Mutex::new(vec![0u8; (FB_WIDTH * FB_HEIGHT * PIXEL_SIZE) as usize])),
                    video_frame: Arc::new(Mutex::new(VideoFrame {
                        rotation: VideoRotation::VideoRotation0,
                        buffer: I420Buffer::new(FB_WIDTH as u32, FB_HEIGHT as u32),
                        timestamp_us: 0,
                    })),
                    pos: (0, 0),
                    direction: (1, 1),
                };

                let duration =  std::time::Duration::from_millis(1000 / 15);
                loop {
        
                    if let Ok(stop) = close_rx.try_recv() {
                        if stop {
                            image_receiver.drain();
                            drop(image_receiver);
                            break;
                        }
                    }
        
                    if let Ok(mut image) = image_receiver.try_recv() {
                        if image.width() > COMPRESSED_WIDTH as u32 || image.height() > COMPRESSED_HEIGHT as u32 {
                            image = image::imageops::resize(&image, COMPRESSED_WIDTH as u32, COMPRESSED_HEIGHT as u32, FilterType::Nearest);
                        }

                        let image_width = image.width();
                        let image_height = image.height();
        
                        data.image = image;
            
                        // let debug_path = std::path::PathBuf::from("src/debug").join("0.png");
                        // data.image.save(debug_path).expect("Failed to save debug image");

                        let raw_image = data.image.as_raw();
                        let mut framebuffer = data.framebuffer.lock();
                        let mut video_frame = data.video_frame.lock();
                        let i420_buffer = &mut video_frame.buffer;
        
                        let (stride_y, stride_u, stride_v) = i420_buffer.strides();
                        let (data_y, data_u, data_v) = i420_buffer.data_mut();
        
                        let logo_stride = image_width as usize * PIXEL_SIZE;
                        framebuffer.fill(0);
                        for i in 0..image_height as usize {
                            let row_start = (data.pos.0 as usize + ((i + data.pos.1 as usize) * FB_WIDTH)) * PIXEL_SIZE;
                            let row_end = row_start + logo_stride;
                            
                            if row_end < framebuffer.len() {
                                framebuffer[row_start..row_end].copy_from_slice(
                                    &raw_image[i * logo_stride..i * logo_stride + logo_stride],
                                );
                            }
                        }
        
                        yuv_helper::abgr_to_i420(
                            &framebuffer,
                            (FB_WIDTH * PIXEL_SIZE) as u32,
                            data_y,
                            stride_y,
                            data_u,
                            stride_u,
                            data_v,
                            stride_v,
                            FB_WIDTH as i32,
                            FB_HEIGHT as i32,
                        );
                        rtc_source.capture_frame(&*video_frame);

                        data.pos.0 += data.direction.0 * MOVE_SPEED;
                        data.pos.1 += data.direction.1 * MOVE_SPEED;
            
                        if data.pos.0 >= (FB_WIDTH as u32 - image_width) as i32 {
                            data.direction.0 = -1;
                        } else if data.pos.0 <= 0 {
                            data.direction.0 = 1;
                        }
            
                        if data.pos.1 >= (FB_HEIGHT as u32 - image_height) as i32 {
                            data.direction.1 = -1;
                        } else if data.pos.1 <= 0 {
                            data.direction.1 = 1;
                        }
                    }
                    std::thread::sleep(duration);
                }
            });
        });
    }
}

impl Drop for DeviceVideoTrack {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.close_sender.send(true);
        }
    }
}