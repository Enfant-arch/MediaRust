use ffmpeg_next::format;
use ffmpeg_next::codec;
use ffmpeg_next::software::scaling::{Context as Scaler, flag::Flags};
use ffmpeg_next::util::frame::video::Video;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Открываем окно для отображения
    let window = video_subsystem.window("Video Player", 800, 600)
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    // Инициализация ffmpeg
    ffmpeg_next::init().unwrap();

    // Открываем видеофайл
    let input_path = "your_video_file.mp4";  // Укажите путь к видео
    let mut format_context = format::input(&input_path)?;

    // Находим видео поток
    let stream_index = format_context
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or("Не удалось найти видео поток")?
        .index();

    let codec_context = codec::Context::from_parameters(format_context.stream(stream_index).unwrap().parameters())?;
    let mut decoder = codec_context.decoder().video()?;

    // Подготовка преобразования цвета для рендеринга с помощью SDL2
    let mut scaler = Scaler::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        PixelFormatEnum::YV12, // SDL формат
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    // Подготовка текстуры для отображения кадров
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::YV12, decoder.width(), decoder.height())?;

    // Чтение и декодирование кадров
    let mut frame = Video::empty();
    for (stream, packet) in format_context.packets() {
        if stream.index() == stream_index {
            if decoder.decode(&packet, &mut frame).is_ok() {
                // Преобразуем кадр к нужному формату
                let mut rgb_frame = Video::empty();
                scaler.run(&frame, &mut rgb_frame)?;

                // Загружаем данные кадра в текстуру
                texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for (i, line) in rgb_frame.plane_data(0).chunks(pitch).enumerate() {
                        buffer[i * pitch..(i + 1) * pitch].copy_from_slice(line);
                    }
                })?;

                // Отображаем текстуру на экране
                canvas.clear();
                canvas.copy(&texture, None, Some(Rect::new(0, 0, decoder.width(), decoder.height())))?;
                canvas.present();

                // Задержка для эмуляции FPS видео
                ::std::thread::sleep(std::time::Duration::from_millis(30));
            }
        }
    }

    Ok(())
}



// todo: 1. Аудио- и видеодекодинг learn  ffmpeg-sys, gstreamer
// todo :  Аудио rodio: Простая библиотека для воспроизведения аудио на Rust. Она не требует FFmpeg или GStreamer, и поддерживает основные аудиоформаты.
// Symphonia: Поддерживает декодирование множества аудиоформатов и предоставляет удобный API для проигрывания.
//3. Видеовывод и рендеринг
// SDL2: С библиотекой rust-sdl2 можно отрисовывать видео и обрабатывать ввод. Это один из самых популярных фреймворков для медиаплееров и игр.
// wgpu или glium: Если вы хотите иметь полный контроль над рендерингом, можно использовать низкоуровневые графические библиотеки, такие как wgpu (поддерживает Vulkan, Metal, Direct3D и OpenGL) или glium.