//! Scripted input for acceptance tests without a human at the mouse.
//!
//! One step per line, `#` starts a comment:
//!
//! ```text
//! wait 2.5                  # seconds
//! point 0 120 40            # cursor onto window pixel (120, 40) of screen 0
//! cursor 800 450            # cursor onto window-of-the-compositor pixel
//! click left                # left | right | middle; press + release
//! press left / release left
//! scroll 3                  # wheel notches, positive = down
//! key ControlLeft+KeyA      # keys pressed in order, released in reverse
//! type hallo welt           # a-z, 0-9, space; uppercase via Shift
//! screenshot out.png
//! windowshot 0 fenster.png    # window texture of screen 0, window pixels
//! quit
//! ```

use std::time::Duration;

use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub enum Step {
    Wait(Duration),
    Point {
        slot: usize,
        x: f64,
        y: f64,
    },
    Cursor {
        x: f64,
        y: f64,
    },
    Click(MouseButton),
    Press(MouseButton),
    Release(MouseButton),
    Scroll(f32),
    Keys(Vec<KeyCode>),
    Screenshot(String),
    /// Window texture of a screen, in window pixels.
    WindowShot {
        slot: usize,
        path: String,
    },
    Quit,
}

pub fn parse(text: &str, swap_yz: bool) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let err = |msg: &str| format!("Zeile {}: {msg}: {line}", number + 1);
        let (command, rest) = line.split_once(' ').unwrap_or((line, ""));
        let nums = || -> Result<Vec<f64>, String> {
            rest.split_whitespace()
                .map(|n| n.parse::<f64>().map_err(|_| err("Zahl erwartet")))
                .collect()
        };
        let step = match command {
            "wait" => Step::Wait(Duration::from_secs_f64(
                *nums()?.first().ok_or_else(|| err("Sekunden fehlen"))?,
            )),
            "point" => match nums()?[..] {
                [slot, x, y] => Step::Point {
                    slot: slot as usize,
                    x,
                    y,
                },
                _ => return Err(err("point SCREEN X Y")),
            },
            "cursor" => match nums()?[..] {
                [x, y] => Step::Cursor { x, y },
                _ => return Err(err("cursor X Y")),
            },
            "click" => Step::Click(button(rest).ok_or_else(|| err("Taste"))?),
            "press" => Step::Press(button(rest).ok_or_else(|| err("Taste"))?),
            "release" => Step::Release(button(rest).ok_or_else(|| err("Taste"))?),
            "scroll" => Step::Scroll(*nums()?.first().ok_or_else(|| err("Rasten fehlen"))? as f32),
            "key" => Step::Keys(
                rest.split('+')
                    .map(|k| key(k.trim()).ok_or_else(|| err("unbekannte Taste")))
                    .collect::<Result<_, _>>()?,
            ),
            "type" => {
                for c in rest.chars() {
                    steps.push(Step::Keys(
                        char_keys(c, swap_yz).ok_or_else(|| err("Zeichen"))?,
                    ));
                }
                continue;
            }
            "screenshot" => Step::Screenshot(rest.to_string()),
            "windowshot" => {
                let (slot, path) = rest
                    .split_once(' ')
                    .ok_or_else(|| err("windowshot SCREEN DATEI"))?;
                Step::WindowShot {
                    slot: slot.parse().map_err(|_| err("Screen-Nummer"))?,
                    path: path.trim().to_string(),
                }
            }
            "quit" => Step::Quit,
            _ => return Err(err("unbekannter Befehl")),
        };
        steps.push(step);
    }
    Ok(steps)
}

fn button(name: &str) -> Option<MouseButton> {
    match name.trim() {
        "left" => Some(MouseButton::Left),
        "right" => Some(MouseButton::Right),
        "middle" => Some(MouseButton::Middle),
        _ => None,
    }
}

const LETTERS: [KeyCode; 26] = [
    KeyCode::KeyA,
    KeyCode::KeyB,
    KeyCode::KeyC,
    KeyCode::KeyD,
    KeyCode::KeyE,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyI,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyM,
    KeyCode::KeyN,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::KeyQ,
    KeyCode::KeyR,
    KeyCode::KeyS,
    KeyCode::KeyT,
    KeyCode::KeyU,
    KeyCode::KeyV,
    KeyCode::KeyW,
    KeyCode::KeyX,
    KeyCode::KeyY,
    KeyCode::KeyZ,
];

const DIGITS: [KeyCode; 10] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

/// Physical keys for one character; `swap_yz` for QWERTZ layouts.
fn char_keys(c: char, swap_yz: bool) -> Option<Vec<KeyCode>> {
    let letter = |c: char| {
        let mut index = (c as u8 - b'a') as usize;
        if swap_yz && (c == 'y' || c == 'z') {
            index = if c == 'y' { 25 } else { 24 };
        }
        LETTERS[index]
    };
    Some(match c {
        'a'..='z' => vec![letter(c)],
        'A'..='Z' => vec![KeyCode::ShiftLeft, letter(c.to_ascii_lowercase())],
        '0'..='9' => vec![DIGITS[(c as u8 - b'0') as usize]],
        ' ' => vec![KeyCode::Space],
        _ => return None,
    })
}

fn key(name: &str) -> Option<KeyCode> {
    if let Some(letter) = name.strip_prefix("Key")
        && letter.len() == 1
    {
        let c = letter.chars().next()?.to_ascii_lowercase();
        return c
            .is_ascii_lowercase()
            .then(|| LETTERS[(c as u8 - b'a') as usize]);
    }
    if let Some(digit) = name.strip_prefix("Digit") {
        return digit
            .parse::<usize>()
            .ok()
            .and_then(|d| DIGITS.get(d).copied());
    }
    Some(match name {
        "Enter" => KeyCode::Enter,
        "Escape" => KeyCode::Escape,
        "Tab" => KeyCode::Tab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Space" => KeyCode::Space,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "ControlLeft" => KeyCode::ControlLeft,
        "ShiftLeft" => KeyCode::ShiftLeft,
        "AltLeft" => KeyCode::AltLeft,
        "F1" => KeyCode::F1,
        "F5" => KeyCode::F5,
        "F10" => KeyCode::F10,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_steps_and_qwertz() {
        let steps = parse(
            "wait 1\npoint 0 10 20\nkey ControlLeft+KeyA\ntype yZ\n",
            true,
        )
        .unwrap();
        assert!(matches!(steps[0], Step::Wait(d) if d == Duration::from_secs(1)));
        assert!(matches!(steps[1], Step::Point { slot: 0, .. }));
        assert!(matches!(&steps[2], Step::Keys(k) if k == &[KeyCode::ControlLeft, KeyCode::KeyA]));
        assert!(matches!(&steps[3], Step::Keys(k) if k == &[KeyCode::KeyZ]));
        assert!(matches!(&steps[4], Step::Keys(k) if k == &[KeyCode::ShiftLeft, KeyCode::KeyY]));
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(parse("jump 3", false).is_err());
    }
}
