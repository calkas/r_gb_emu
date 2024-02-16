use minifb::{Key, Window};
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

    let mut gameboy = GameBoyEmulator::default();
    gameboy.load_cartridge("roms/07-jr,jp,call,ret,rst.gb");

    let mut frame_buffer: Vec<u32> = vec![0x348feb; resolution::SCREEN_W * resolution::SCREEN_H];

    let window_option = minifb::WindowOptions {
        resize: true,
        scale: minifb::Scale::X4,
        ..Default::default()
    };
    let window_name = String::from("r_gb_emu - ") + gameboy.cartridge_name();

    let mut window = Window::new(
        &window_name,
        resolution::SCREEN_W,
        resolution::SCREEN_H,
        window_option,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        gameboy.emulate_frame(frame_buffer.as_mut_slice());

        window
            .update_with_buffer(&frame_buffer, resolution::SCREEN_W, resolution::SCREEN_H)
            .unwrap();

        keyboard_handle_event(&window, &mut gameboy);
    }

    println!(
        "\x1b[96m=========================\n      ..::END::..      \n=========================\x1b[0m"
    );
}
