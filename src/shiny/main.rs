#[feature(globs)];

extern mod stb;

extern mod glfw;
extern mod opengles;
extern mod portaudio;

use opengles::gl2;
use portaudio::*;

use std::num::*;
use std::f32;

//use std::io::File;

fn sinewave() {
    let bufsize = 1024;

    println!("Portaudio init error : {:s}", pa::get_error_text(pa::initialize()));
    let def_output = pa::get_default_output_device();
    let info_output = pa::get_device_info(def_output).unwrap();
    println!("Default output device info :");
    println!("version : {:d}", info_output.struct_version);
    println!("name : {:s}", info_output.name);
    println!("max output channels : {:d}", info_output.max_output_channels);
    println!("max output channels : {:d}", info_output.max_output_channels);
    println!("default sample rate : {:f}", info_output.default_sample_rate);

    let isr = 1.0 / info_output.default_sample_rate as f32;

    do spawn {
        let stream_params_out = types::PaStreamParameters {
            device : def_output,
            channel_count : 1,
            sample_format : types::PaFloat32,
            suggested_latency : pa::get_device_info(def_output).unwrap().default_low_output_latency
        };

        let mut stream : pa::PaStream<f32> = pa::PaStream::new(types::PaFloat32);

        let mut err= stream.open(None, Some(&stream_params_out), 44100., 1024, types::PaClipOff);
        println!("Portaudio Open error : {:s}", pa::get_error_text(err));

        err = stream.start();
        println!("Portaudio Start error : {:s}", pa::get_error_text(err));

        let mut phase = 0.0;
        loop{
            let mut buf:~[f32] = ~[];
            buf.reserve(bufsize);
            buf.grow_fn(bufsize, |_|{
                phase += f32::consts::PI * 440.0 * isr;
                sin(phase)
            });
            stream.write(buf, bufsize as u32);
        }
    }
}

pub fn main() {
    // XXX: Not cleaned up when main exits.
    spawn(sinewave);

    println!("Shiny: A prototype user interface.");
    do glfw::start {
        let window = glfw::Window::create(800, 600, "Shiny!", glfw::Windowed)
            .expect("Failed to create window.");
        window.make_context_current();

	gl2::viewport(0, 0, 800, 600);
	gl2::clear_color(0.0, 0.8, 0.8, 1.0);
	gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);
	gl2::flush();

	window.swap_buffers();

        while !window.should_close() {
            glfw::poll_events();
        }
    }
    /*
    let font = stb::truetype::Font::new(
        File::open(&Path::new("assets/pf_tempesta_seven_extended_bold.ttf")).read_to_end())
        .unwrap();

    let glyph = font.glyph(80, 13.0).unwrap();

    for y in range(0, glyph.height) {
        for x in range(0, glyph.width) {
            app.pixels[x * 4 + y * 4 * 800 + 1] = glyph.pixels[x + y * glyph.width];
        }
    }
    */
}
