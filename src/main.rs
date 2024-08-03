use clap::Parser;
use ffmpeg_next::{Error, format, Packet};
use ffmpeg_next::format::context::Input;
use ffmpeg_next::format::Pixel;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type;
use ffmpeg_next::rescale::TIME_BASE;
use ffmpeg_next::software::scaling::{Context, Flags};
use image::{Rgb, RgbImage};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    quality: u32,

    #[arg(short, long)]
    time: u32,

    #[arg(short, long)]
    size: u32,

    #[arg(short, long)]
    codec: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    ffmpeg_next::init()?;
    let mut ictx = format::input(&args.input)?;
    let input = ictx
        .streams()
        .best(Type::Video)
        .ok_or(Error::StreamNotFound)?;
    let stream_index = input.index();

    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let scaler = create_scaler(&args, &mut decoder)?;

    let seek_percent = args.time as f64 * 0.01;
    seek_to_position(&mut ictx, seek_percent)?;

    let rgb_frame = get_frame(ictx, stream_index, decoder, scaler)?;

    if rgb_frame != Video::empty() {
        if let Err(e) = write_frame_to_jpeg(&rgb_frame, &args.output) {
            eprintln!("Error writing file {:?}", e);
        }
    } else {
        eprintln!("Could not find frame");
    }

    Ok(())
}

fn seek_to_position(ictx: &mut Input, seek_percent: f64) -> Result<(), Error> {
    let seek_pos_in_seconds = ((ictx.duration() as f64 * seek_percent) / f64::from(TIME_BASE.denominator())) as i32;
    let seek_pos = seek_pos_in_seconds * TIME_BASE.denominator();
    ictx.seek(seek_pos as i64, ..seek_pos as i64)?;
    Ok(())
}

fn create_scaler(args: &Args,  decoder: &mut ffmpeg_next::codec::decoder::video::Video) -> Result<Context, Error> {
    let w = decoder.width();
    let p = args.size as f32 / w as f32;
    let h = decoder.height();
    let new_h = (h as f32 * p) as u32;
    let scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        args.size,
        new_h,
        Flags::BILINEAR,
    )?;
    Ok(scaler)
}

fn get_frame(mut ictx: Input,
             stream_index: usize,
             mut decoder: ffmpeg_next::codec::decoder::video::Video,
             mut scaler: Context) -> Result<Video, Error> {
    let mut rgb_frame = Video::empty();
    for (stream, packet) in ictx.packets() {
        if stream.index() == stream_index {
            if let Ok(frame_decoded) = process_packet(&mut decoder, &mut scaler, &packet, &mut rgb_frame) {
                if frame_decoded {
                    break;
                }
            }
        }
    }

    Ok(rgb_frame)
}

fn write_frame_to_jpeg(frame: &Video, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    if frame.format() != Pixel::RGB24 {
        return Err("Frame format is not RGB24".into());
    }

    let width = frame.width();
    let height = frame.height();

    let mut img_buffer = RgbImage::new(width, height);

    let data = frame.data(0);
    let linesize = frame.stride(0);

    for y in 0..height {
        for x in 0..width {
            let offset = (y * linesize as u32 + x * 3) as usize;
            let rgb = Rgb([data[offset], data[offset + 1], data[offset + 2]]);
            img_buffer.put_pixel(x, y, rgb);
        }
    }

    img_buffer.save_with_format(filename, image::ImageFormat::Jpeg)?;

    Ok(())
}

fn process_packet(
    decoder: &mut ffmpeg_next::decoder::Video,
    scaler: &mut Context,
    packet: &Packet,
    rgb_frame: &mut Video,
) -> Result<bool, Error> {
    decoder.send_packet(packet)?;
    let mut decoded = Video::empty();
    match decoder.receive_frame(&mut decoded) {
        Ok(_) => {
            scaler.run(&decoded, rgb_frame)?;
            Ok(true)
        }
        Err(err) => {
            eprintln!("Failed to receive frame: {:?}", err);
            Ok(false)
        },
    }
}
