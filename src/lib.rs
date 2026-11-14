#![allow(static_mut_refs)]
#[cfg(feature = "buddy-alloc")]
mod alloc;
mod text;
mod wasm4;
use heapless::index_map::FnvIndexMap;

include!(concat!(env!("OUT_DIR"), "/story.rs"));

use wasm4::*;

const SPRITE_SIZE: u32 = 16;
enum Mode {
    Title,
    Main,
    Popup,
    Combat,
    Character,
}

struct GameState {
    mode: Mode,
    selected_option: usize,
    sections: FnvIndexMap<usize, Section, 128>,
    active_section: usize,
    player_name: &'static str,
    items: Vec<&'static str>,
    hp: u32,
    max_hp: u32,
}

struct Section {
    id: usize,
    title: &'static str,
    description: &'static str,
    options: Vec<Option>,
}

struct Option {
    label: &'static str,
    target_section: usize,
}

static mut GAME: GameState = GameState {
    mode: Mode::Main,
    selected_option: 0,
    sections: FnvIndexMap::new(),
    active_section: 0,
    player_name: "Fearless Steve",
    items: Vec::new(),
    hp: 100,
    max_hp: 100,
};
static mut PREVIOUS_GAMEPAD: u8 = 0;

#[no_mangle]
fn start() {
    // Set the color palette for the game
    unsafe {
        *PALETTE = [0x99c9b3, 0x579c9a, 0x294257, 0x191b1a];

        // This bit of magic gives me readable errors when code panics
        std::panic::set_hook(Box::new(|info| {
            wasm4::trace(format!("{info}"));
        }));

        load_story(); // Comes from the include of story.rs above

        // For debugging
        GAME.active_section = 0;
    }
}

#[no_mangle]
fn update() {
    unsafe {
        let (pressed_this_frame, ..) = {
            let previous = PREVIOUS_GAMEPAD;
            let gamepad = *GAMEPAD1;
            // Only the buttons that were pressed down this frame
            let pressed_this_frame = gamepad & (gamepad ^ previous);
            PREVIOUS_GAMEPAD = gamepad;

            (pressed_this_frame, gamepad, previous)
        };

        let active_section = GAME.active_section;
        let section = &GAME.sections[&active_section];

        *DRAW_COLORS = 0x30;
        text::draw_text(section.title, 4, 2);

        *DRAW_COLORS = 0x40;
        text::draw_text(section.description, 1, 15);

        *DRAW_COLORS = 0x2;
        line(2, 12, 157, 12);

        // six option buttons for the current section
        for (i, option) in section.options.iter().enumerate() {
            draw_beveled_button(4, 100 + i as i32 * 20, 150, 16, option.label, i == GAME.selected_option);
        }

        let option_count = GAME.sections[&GAME.active_section].options.len();

        if pressed_this_frame & BUTTON_DOWN != 0 && option_count > 0 {
            if GAME.selected_option >= option_count - 1 {
                GAME.selected_option = option_count - 1
            } else {
                GAME.selected_option += 1;
            }
        }

        if pressed_this_frame & BUTTON_UP != 0 {
            if GAME.selected_option == 0 {
                GAME.selected_option = 0;
            } else {
                GAME.selected_option -= 1;
            }
        }

        if pressed_this_frame & BUTTON_1 != 0 && option_count > 0 {
            let selected_option = &GAME.sections[&GAME.active_section].options[GAME.selected_option];
            // check exists
            if GAME.sections.contains_key(&selected_option.target_section) {
                GAME.active_section = selected_option.target_section;
                GAME.selected_option = 0;
                tone(162 | (223 << 16), 10, 100, TONE_MODE4);
            }
        }
    }
}

// Draws a raised button with a two-pixel light/dark bevel.
pub fn draw_beveled_button(x: i32, y: i32, width: u32, height: u32, label: &str, select: bool) {
    unsafe {
        // Fill center
        *DRAW_COLORS = 0x1;
        rect(x, y, width, height);

        // Light bevel on top and left
        *DRAW_COLORS = if select { 0x3 } else { 0x2 };
        line(x, y, x + width as i32, y);
        line(x, y + 1, x + width as i32, y + 1);
        line(x, y, x, y + height as i32);
        line(x + 1, y, x + 1, y + height as i32);

        // Dark bevel on bottom and right
        *DRAW_COLORS = if select { 0x2 } else { 0x3 };
        line(x, y + height as i32, x + width as i32, y + height as i32);
        line(x + 1, y + height as i32 - 1, x + width as i32, y + height as i32 - 1);
        line(x + width as i32 + 1, y, x + width as i32 + 1, y + height as i32);
        line(x + width as i32, y + 1, x + width as i32, y + height as i32);

        // Text label
        *DRAW_COLORS = 0x40;
        let label_string = if select {
            format!(">{label}")
        } else {
            format!("{label}")
        };

        text::draw_text(&label_string, x + 4, y + 4);
    }
}

pub fn draw_sprite(sprite: &[u8; 64], x: i32, y: i32) {
    unsafe {
        *DRAW_COLORS = 0x04321;
    }

    let x_flip_chance = true; //rand() % 2 == 0;

    blit(
        sprite,
        x * SPRITE_SIZE as i32 + 93,
        y * SPRITE_SIZE as i32 + 3,
        SPRITE_SIZE,
        SPRITE_SIZE,
        BLIT_2BPP | if x_flip_chance { BLIT_FLIP_X } else { 0 },
    );
}
