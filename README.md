# r_gb_emu

[![Rust](https://img.shields.io/badge/language-rust-maroon)](https://rust-lang.org/)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

A Gameboy emulator 🎮

## Useful Links
### Docs
- [A journey into Gameboy Emulation Blog](https://robertovaccari.com/blog/2020_09_26_gameboy/)
- [Codeslinger](http://www.codeslinger.co.uk/pages/projects/gameboy/lcd.html)
- [EmuDev](https://emudev.de/gameboy-emulator/%e2%af%88-ppu-rgb-arrays-and-sdl/)
- [Pandocs](https://gbdev.io/pandocs/CPU_Instruction_Set.html#8-bit-load-instructions)
- [DMG-01: How to Emulate a Game Boy](https://rylev.github.io/DMG-01/public/book/introduction.html)
- [8080 Programmers Manual](https://altairclone.com/downloads/manuals/8080%20Programmers%20Manual.pdf)

- [Everything You Always Wanted To Know About GAMEBOY](https://www.devrs.com/gb/files/gbspec.txt)
- [Study of the techniques for emulation 
programming](http://www.codeslinger.co.uk/files/emu.pdf)

- [Nes emulator](https://bugzmanov.github.io/nes_ebook/chapter_2.html)

### Repos
- [rboy](https://github.com/mvdnes/rboy/tree/master)
- [rustyboy](https://github.com/daveallie/rustyboy)
- [mohanson](https://github.com/mohanson/gameboy)

### Libs
- [minifb](https://crates.io/crates/minifb)


### Debugging
- [Gameboy doctor](https://github.com/robert/gameboy-doctor?tab=readme-ov-file)

Once you have your logfile (by running `07-jr,jp,call,ret,rst.gb`), feed it into Gameboy Doctor like so:

```bash
./gameboy-doctor cpu_log.log  cpu_instrs 7
```