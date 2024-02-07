use minifb::{Key, Window, WindowOptions};
use r_gb_emu::emulator_constants::{resolution, GameBoyKeys};
use r_gb_emu::GameBoyEmulator;

fn keyboard_handle_event(window: &Window, gameboy: &mut GameBoyEmulator) {
    let key_map: [(Key, GameBoyKeys); 8] = [
        (minifb::Key::Right, GameBoyKeys::Right),
        (minifb::Key::Up, GameBoyKeys::Up),
        (minifb::Key::Left, GameBoyKeys::Left),
        (minifb::Key::Down, GameBoyKeys::Down),
        (minifb::Key::Z, GameBoyKeys::A),
        (minifb::Key::X, GameBoyKeys::B),
        (minifb::Key::Space, GameBoyKeys::Select),
        (minifb::Key::Enter, GameBoyKeys::Start),
    ];

    for (frame_work_key, emulator_key) in key_map {
        if window.is_key_pressed(frame_work_key, minifb::KeyRepeat::Yes) {
            gameboy.button_pressed(emulator_key);
        } else {
            gameboy.button_released(emulator_key);
        }
    }
}

fn main() {
    println!("\x1b[94m=========================\n..::Gameboy Emulator::..\n=========================\x1b[0m");

    let mut gameboy = GameBoyEmulator::new();
    const WIDTH: usize = resolution::SCREEN_W;
    const HEIGHT: usize = resolution::SCREEN_H;

    let frame_buffer: Vec<u32> = vec![0x348feb; WIDTH * HEIGHT];

    let mut window = Window::new("r_gb_emu", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&frame_buffer, WIDTH, HEIGHT)
            .unwrap();

        keyboard_handle_event(&window, &mut gameboy);
    }

    println!(
        "\x1b[96m=========================\n      ..::END::..      \n=========================\x1b[0m"
    );
}
