//! Recruitment state - hire new adventurers

use super::StateTransition;
use crate::kingdom::{
    evaluate_recruitment, recruit_cost, Adventurer, AdventurerClass, KingdomState, Roster,
};
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::rng;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

/// Names for random adventurers
const FIRST_NAMES: &[&str] = &[
    "Aldric", "Beatrix", "Cedric", "Diana", "Edmund", "Freya", "Godric", "Helena", "Ivan",
    "Jocelyn", "Klaus", "Lydia", "Magnus", "Nadia", "Oscar", "Petra", "Quinn", "Rosa", "Stefan",
    "Thea", "Ulric", "Vera", "Werner", "Xena",
];

/// A recruit available for hire
#[derive(Clone)]
pub struct Recruit {
    pub adventurer: Adventurer,
    pub cost: i32,
}

impl Recruit {
    pub fn random(class: AdventurerClass) -> Self {
        let name = FIRST_NAMES[rng::gen_range(0, FIRST_NAMES.len())];
        let gender = if rng::gen_range(0, 2) == 0 {
            crate::kingdom::Gender::Male
        } else {
            crate::kingdom::Gender::Female
        };

        let cost = recruit_cost(&class);
        let adventurer = Adventurer::new(name, class, gender);
        Self { adventurer, cost }
    }
}

/// State for recruiting new adventurers
pub struct RecruitState {
    pub recruits: Vec<Recruit>,
    pub selected: usize,
}

impl Default for RecruitState {
    fn default() -> Self {
        Self::new()
    }
}

impl RecruitState {
    pub fn new() -> Self {
        // Generate 3 random recruits
        let recruits = vec![
            Recruit::random(AdventurerClass::Soldier),
            Recruit::random(AdventurerClass::Scout),
            Recruit::random(AdventurerClass::Healer),
        ];

        Self {
            recruits,
            selected: 0,
        }
    }

    pub fn update(
        &mut self,
        kingdom: &mut KingdomState,
        roster: &mut Roster,
    ) -> Option<StateTransition> {
        // Selection
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && self.selected > 0 {
            self.selected -= 1;
        }
        if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S))
            && self.selected < self.recruits.len().saturating_sub(1)
        {
            self.selected += 1;
        }

        // Number keys
        for i in 0..self.recruits.len().min(9) {
            let key = match i {
                0 => KeyCode::Key1,
                1 => KeyCode::Key2,
                2 => KeyCode::Key3,
                _ => continue,
            };
            if is_key_pressed(key) {
                self.selected = i;
            }
        }

        for i in 0..self.recruits.len() {
            let (x, y, w, h) = recruit_card_rect(i);
            if crate::ui::was_clicked(x, y, w, h) {
                if self.selected == i {
                    self.hire_selected(kingdom, roster);
                } else {
                    self.selected = i;
                }
            }
        }

        if crate::ui::was_clicked(
            hire_button_rect().0,
            hire_button_rect().1,
            hire_button_rect().2,
            hire_button_rect().3,
        ) {
            self.hire_selected(kingdom, roster);
        }

        // Hire with Enter
        if is_key_pressed(KeyCode::Enter) {
            self.hire_selected(kingdom, roster);
        }

        if crate::ui::was_clicked(
            back_button_rect().0,
            back_button_rect().1,
            back_button_rect().2,
            back_button_rect().3,
        ) || is_key_pressed(KeyCode::Escape)
        {
            return Some(StateTransition::ToBase);
        }

        None
    }

    fn hire_selected(&mut self, kingdom: &mut KingdomState, roster: &mut Roster) {
        let Some(recruit) = self.recruits.get(self.selected) else {
            return;
        };
        let evaluation = evaluate_recruitment(kingdom.stats.gold, roster.count(), recruit.cost);
        if !evaluation.can_hire {
            return;
        }

        kingdom.stats.gold -= recruit.cost;
        roster.add(recruit.adventurer.clone());
        self.recruits.remove(self.selected);
        if self.selected >= self.recruits.len() && self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn draw(
        &self,
        kingdom: &KingdomState,
        roster: &Roster,
        textures: &HashMap<String, Texture2D>,
    ) {
        draw_background(
            textures,
            BackgroundArt::Recruit,
            Color::from_rgba(5, 5, 8, 174),
        );
        draw_ui_text(
            "RECRUITMENT",
            20.0,
            40.0,
            32.0,
            Color::from_rgba(236, 224, 198, 255),
        );
        draw_icon(textures, SpriteIcon::Gold, 18.0, 48.0, 26.0, WHITE);
        draw_ui_text(
            &format!("Gold: {}", kingdom.stats.gold),
            52.0,
            70.0,
            20.0,
            Color::from_rgba(214, 154, 62, 255),
        );

        let start_y = 120.0;
        let card_height = 120.0;
        let card_width = 500.0;

        for (i, recruit) in self.recruits.iter().enumerate() {
            let (_, y, _, _) = recruit_card_rect(i);
            let is_selected = i == self.selected;
            let can_afford = kingdom.stats.gold >= recruit.cost;
            // ... (rest of helper) ...

            // Background
            let bg_color = if is_selected {
                if can_afford {
                    Color::from_rgba(76, 59, 32, 245)
                } else {
                    Color::from_rgba(69, 39, 33, 245)
                }
            } else {
                Color::from_rgba(27, 23, 21, 232)
            };
            draw_rectangle(20.0, y, card_width, card_height, bg_color);

            if is_selected {
                let border = if can_afford {
                    Color::from_rgba(214, 154, 62, 255)
                } else {
                    Color::from_rgba(150, 55, 48, 255)
                };
                draw_rectangle_lines(20.0, y, card_width, card_height, 2.0, border);
            }

            // Portrait
            if let Some(path) = &recruit.adventurer.image_path {
                if let Some(tex) = textures.get(path) {
                    draw_texture_ex(
                        tex,
                        30.0,
                        y + 10.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(100.0, 100.0)),
                            ..Default::default()
                        },
                    );
                }
            }

            let class_icon = match recruit.adventurer.class {
                AdventurerClass::Soldier => SpriteIcon::Attack,
                AdventurerClass::Scout => SpriteIcon::Knowledge,
                AdventurerClass::Healer => SpriteIcon::Vitality,
                AdventurerClass::Mystic => SpriteIcon::Event,
            };
            draw_icon(textures, class_icon, 450.0, y + 12.0, 48.0, WHITE);

            // Info
            let text_color = if is_selected {
                Color::from_rgba(236, 224, 198, 255)
            } else {
                Color::from_rgba(164, 153, 130, 255)
            };
            draw_ui_text(
                &format!("[{}] {}", i + 1, recruit.adventurer.name),
                140.0,
                y + 30.0,
                22.0,
                text_color,
            );
            draw_ui_text(
                &format!(
                    "{:?} - {:?}",
                    recruit.adventurer.gender, recruit.adventurer.class
                ),
                140.0,
                y + 55.0,
                18.0,
                Color::from_rgba(164, 153, 130, 255),
            );
            draw_ui_text(
                &format!("HP: {}", recruit.adventurer.max_hp),
                140.0,
                y + 80.0,
                16.0,
                Color::from_rgba(112, 143, 92, 255),
            );

            // Cost
            let cost_color = if can_afford {
                Color::from_rgba(214, 154, 62, 255)
            } else {
                Color::from_rgba(150, 55, 48, 255)
            };
            draw_ui_text(
                &format!("Cost: {} Gold", recruit.cost),
                350.0,
                y + 55.0,
                18.0,
                cost_color,
            );
        }

        if self.recruits.is_empty() {
            draw_ui_text(
                "No recruits available",
                20.0,
                start_y + 30.0,
                24.0,
                Color::from_rgba(164, 153, 130, 255),
            );
        }

        draw_ui_text(
            &format!(
                "Roster: {}/{}",
                roster.count(),
                crate::kingdom::MAX_ROSTER_SIZE
            ),
            20.0,
            96.0,
            16.0,
            Color::from_rgba(164, 153, 130, 255),
        );

        let (hire_x, hire_y, hire_w, hire_h) = hire_button_rect();
        draw_action_button(
            "Hire Selected",
            hire_x,
            hire_y,
            hire_w,
            hire_h,
            self.recruits.get(self.selected).is_some_and(|recruit| {
                evaluate_recruitment(kingdom.stats.gold, roster.count(), recruit.cost).can_hire
            }),
        );
        let (back_x, back_y, back_w, back_h) = back_button_rect();
        draw_action_button("Back", back_x, back_y, back_w, back_h, true);
        draw_ui_text(
            "Tap a recruit to select • Tap HIRE SELECTED to hire • Tap BACK to return",
            20.0,
            screen_height() - 40.0,
            16.0,
            Color::from_rgba(112, 143, 92, 255),
        );
    }
}

fn recruit_card_rect(i: usize) -> (f32, f32, f32, f32) {
    (20.0, 120.0 + (i as f32 * 130.0), 500.0, 120.0)
}

fn hire_button_rect() -> (f32, f32, f32, f32) {
    (screen_width() - 300.0, 22.0, 136.0, 36.0)
}

fn back_button_rect() -> (f32, f32, f32, f32) {
    (screen_width() - 152.0, 22.0, 126.0, 36.0)
}

fn draw_action_button(label: &str, x: f32, y: f32, w: f32, h: f32, enabled: bool) {
    let hovered = crate::ui::is_mouse_over(x, y, w, h);
    let fill = if !enabled {
        Color::from_rgba(31, 27, 25, 218)
    } else if hovered {
        Color::from_rgba(111, 75, 32, 245)
    } else {
        Color::from_rgba(70, 49, 27, 238)
    };
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.0, if enabled { GOLD } else { GRAY });
    let text_width = measure_ui_text(label, None, 16, 1.0).width;
    draw_ui_text(
        label,
        x + (w - text_width) / 2.0,
        y + 24.0,
        16.0,
        if enabled { WHITE } else { GRAY },
    );
}
