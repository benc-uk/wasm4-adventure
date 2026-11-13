#[cfg(feature = "buddy-alloc")]
mod alloc;
mod text;
mod wasm4;
use heapless::index_map::FnvIndexMap;

use wasm4::*;

const SPRITE_SIZE: u32 = 16;

struct GameState {
    selected_option: usize,
    sections: FnvIndexMap<usize, Section, 128>,
    active_section: usize,
}

struct Section {
    id: usize,
    title: String,
    description: String,
    options: Vec<Option>,
}

struct Option {
    label: String,
    target_section: usize,
}

static mut GAME: GameState = GameState {
    selected_option: 0,
    sections: FnvIndexMap::new(),
    active_section: 0,
};
static mut PREVIOUS_GAMEPAD: u8 = 0;

#[no_mangle]
fn start() {
    // Set the color palette for the game
    unsafe {
        *PALETTE = [0x99c9b3, 0x579c9a, 0x294257, 0x191b1a];

        // Create the initial sections for the game
        let mut section = Section {
            title: "Woodland Path".to_string(),
            description: "You are tired from your journey\nand stand in some woodland,\nby a path running north. \n\nThere is a small, tatty wooden\nhut.".to_string(),
            options: vec![
                Option {
                    label: "Go North".to_string(),
                    target_section: 3,
                },
                Option {
                    label: "Enter Hut".to_string(),
                    target_section: 1,
                },
                Option {
                    label: "Go South".to_string(),
                    target_section: 4,
                },
            ],
            id: 0,
        };
        _ = GAME.sections.insert(section.id, section);

        section = Section {
            title: "Inside Hut".to_string(),
            description: "Inside the hut.\nIt is very tatty and run down\n\nThere is a small table in the\ncorner, a notebook lies on it.".to_string(),
            options: vec![Option {
                label: "Leave".to_string(),
                target_section: 0,
            },
            Option {
                label: "Read Notebook".to_string(),
                target_section: 2,
            }],
            id: 1,
        };
        _ = GAME.sections.insert(section.id, section);

        section = Section {
            title: "Dusty Notebook".to_string(),
            description: "The notebook is dusty and old.\nIts pages filled with ancient\nwritings.\n\nYou open to a page that\nsimply says:\n\"Phil Smells of cabbage farts\".".to_string(),
            options: vec![
                Option {
                    label: "Return to Hut".to_string(),
                    target_section: 1,
                },
            ],
            id: 2,
        };
        _ = GAME.sections.insert(section.id, section);

        section = Section {
            title: "The Woods".to_string(),
            description: "You are in the woods.\nThe trees are tall and dense,\nblocking most of the sunlight.\n\nYou are too scared to go\nfurther.".to_string(),
            options: vec![
                Option {
                    label: "Scarper".to_string(),
                    target_section: 0,
                },
            ],
            id: 3,
        };
        _ = GAME.sections.insert(section.id, section);

        section = Section {
            title: "A Meandering Path".to_string(),
            description: "The path winds through the\nforest, surrounded by tall trees\nand dense foliage.\n\nYou can go no further, as Ben\nhasn't added the rest of\nthe zgame code".to_string(),
            options: vec![Option {
                label: "Return".to_string(),
                target_section: 0,
            }],
            id: 4,
        };
        _ = GAME.sections.insert(section.id, section);

        // For debugging
        GAME.active_section = 4;
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
        text::draw_text(&section.title, 4, 2);

        *DRAW_COLORS = 0x40;
        text::draw_text(&section.description, 1, 15);

        *DRAW_COLORS = 0x2;
        line(2, 12, 157, 12);

        // six option buttons for the current section
        for (i, option) in section.options.iter().enumerate() {
            draw_beveled_button(
                4,
                100 + i as i32 * 20,
                150,
                16,
                &option.label,
                i == unsafe { GAME.selected_option },
            );
        }

        if pressed_this_frame & BUTTON_DOWN != 0 {
            if GAME.selected_option >= GAME.sections[&GAME.active_section].options.len() - 1 {
                GAME.selected_option = GAME.sections[&GAME.active_section].options.len() - 1
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

        if pressed_this_frame & BUTTON_2 != 0 {
            let selected_option = &GAME.sections[&GAME.active_section].options[GAME.selected_option];
            // check exists
            if GAME.sections.contains_key(&selected_option.target_section) {
                GAME.active_section = selected_option.target_section;
                GAME.selected_option = 0;
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
