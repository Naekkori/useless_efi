#![no_main]
#![no_std]

extern crate alloc;

use uefi::prelude::*;
use uefi::proto::console::gop::{BltOp, BltPixel, GraphicsOutput};
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};

#[entry]
fn main() -> Status {
    let _ = uefi::helpers::init();

    // GOP (Graphics Output Protocol) 초기화
    let gop_handle =
        boot::get_handle_for_protocol::<GraphicsOutput>().expect("GOP handle not found");
    let mut gop =
        boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).expect("GOP not found");

    // 최고 해상도 모드 선택
    let mut max_res = 0;
    let mut best_mode = None;
    for mode in gop.modes() {
        let (width, height) = mode.info().resolution();
        if width * height > max_res {
            max_res = width * height;
            best_mode = Some(mode);
        }
    }
    if let Some(mode) = best_mode {
        gop.set_mode(&mode).expect("Failed to set graphics mode");
    }

    let (width, height) = gop.current_mode_info().resolution();

    // 화면 초기화 (검은색)
    gop.blt(BltOp::VideoFill {
        color: BltPixel::new(0, 0, 0),
        dest: (0, 0),
        dims: (width, height),
    })
    .expect("Failed to clear screen");

    // 5. 파일 열기
    let file_handle = match boot::get_image_file_system(boot::image_handle())
        .expect("Failed to get image file system")
        .open_volume()
        .expect("Failed to open volume")
        .open(
            cstr16!("\\message.txt"), // 루트 경로 명시
            FileMode::Read,
            FileAttribute::empty(),
        ) {
        Ok(h) => h,
        Err(e) => {
            // 파일이 없으면 화면 전체를 빨간색으로 채워 시각적으로 알림
            let (w, h) = gop.current_mode_info().resolution();
            gop.blt(BltOp::VideoFill {
                color: BltPixel::new(255, 0, 0),
                dest: (0, 0),
                dims: (w, h),
            })
            .ok();
            boot::stall(core::time::Duration::from_secs(5));
            return Status::ABORTED;
        }
    };

    match file_handle.into_type().expect("type conversion failed") {
        FileType::Regular(mut file) => {
            let mut buffer = [0u8; 32768];
            let bytes_read = file.read(&mut buffer).expect("file read failed");
            let valid_len = match core::str::from_utf8(&buffer[..bytes_read]) {
                Ok(_) => bytes_read,
                Err(e) => e.valid_up_to(),
            };
            let content = core::str::from_utf8(&buffer[..valid_len]).unwrap_or("");
            let mut textbuff = alloc::vec::Vec::new();
            for l in content.lines() {
                textbuff.push(l);
            }

            let rect_size = 4;
            let start_x = 20;
            let (width, _height) = gop.current_mode_info().resolution();

            // 상단 100픽셀 지점부터 800픽셀 높이의 영역만 사용
            let region_height = 800;
            let region_top = 100;
            let draw_y = region_top + region_height - rect_size;

            loop {
                for (i, l) in textbuff.iter().enumerate() {
                    let _ = gop.blt(BltOp::VideoToVideo {
                        src: (0, region_top + rect_size),
                        dest: (0, region_top),
                        dims: (width, region_height - rect_size),
                    });

                    let _ = gop.blt(BltOp::VideoFill {
                        color: BltPixel::new(0, 0, 0),
                        dest: (0, draw_y),
                        dims: (width, rect_size),
                    });

                    let mut x = start_x;
                    let mut block_start = None;
                    for c in l.chars() {
                        if c == '■' {
                            if block_start.is_none() {
                                block_start = Some(x);
                            }
                        } else {
                            if let Some(start) = block_start {
                                let _ = gop.blt(BltOp::VideoFill {
                                    color: BltPixel::new(255, 255, 255),
                                    dest: (start, draw_y),
                                    dims: (x - start, rect_size),
                                });
                                block_start = None;
                            }
                        }
                        x += rect_size;
                        if x + rect_size > width {
                            break;
                        }
                    }
                    if let Some(start) = block_start {
                        let _ = gop.blt(BltOp::VideoFill {
                            color: BltPixel::new(255, 255, 255),
                            dest: (start, draw_y),
                            dims: (x - start, rect_size),
                        });
                    }

                    boot::stall(core::time::Duration::from_millis(5));
                }
                boot::stall(core::time::Duration::from_secs(2));
            }
        }
        _ => Status::ABORTED,
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
