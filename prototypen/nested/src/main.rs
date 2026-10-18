//! M0 prototype: nested Wayland compositor that shows client windows as
//! textures on tilted quads in a free-flying 3D scene.
//!
//! Controls: left click captures the mouse, Escape releases it, WASD moves,
//! Space/Shift go up/down, Ctrl moves faster, F12 saves a screenshot.

mod camera;
mod compositor;
mod dmabuf;
mod render;
mod script;
mod window;

use std::collections::{HashSet, VecDeque};
use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use glam::{Vec2, Vec3};
use rustix::event::{PollFd, PollFlags};
use smithay::input::pointer::CursorImageStatus;
use winit::application::ApplicationHandler;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::window::{CursorGrabMode, CursorIcon, Window, WindowId};

use camera::Camera;
use compositor::{Shell, Target};
use render::Renderer;
use script::Step;

const USAGE: &str = "\
nested [--spawn BEFEHL]... [--screenshot DATEI] [--screenshot-delay SEK] [--exit-after-screenshot]
       [--camera X,Y,Z,GIER,NICK] [--script DATEI]

  --spawn BEFEHL            BEFEHL (an Leerzeichen getrennt) als Programm in diesem Compositor starten
  --screenshot DATEI        PNG der Szene nach der Wartezeit speichern
  --screenshot-delay SEK    Wartezeit für --screenshot (Standard 3)
  --exit-after-screenshot   nach dem Screenshot beenden
  --camera X,Y,Z,GIER,NICK  Startposition, Winkel in Grad (Standard 0,1.6,3.2,0,-3)
  --script DATEI            Eingabe-Schritte aus DATEI abspielen (Format: src/script.rs)

Steuerung: Linksklick fängt die Maus, Esc gibt sie frei, WASD bewegt,
Leertaste/Shift hoch/runter, Strg schneller, F12 speichert einen Screenshot.";

struct Args {
    spawn: Vec<String>,
    screenshot: Option<PathBuf>,
    screenshot_delay: Duration,
    exit_after_screenshot: bool,
    camera: [f32; 5],
    script: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        spawn: Vec::new(),
        screenshot: None,
        screenshot_delay: Duration::from_secs(3),
        exit_after_screenshot: false,
        camera: [0.0, 1.6, 3.2, 0.0, -3.0],
        script: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or(format!("{arg} braucht einen Wert"));
        match arg.as_str() {
            "--spawn" => args.spawn.push(value()?),
            "--screenshot" => args.screenshot = Some(value()?.into()),
            "--screenshot-delay" => {
                let secs: f64 = value()?.parse().map_err(|e| format!("{e}"))?;
                args.screenshot_delay = Duration::from_secs_f64(secs);
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            "--exit-after-screenshot" => args.exit_after_screenshot = true,
            "--script" => args.script = Some(value()?.into()),
            "--camera" => {
                let parts: Vec<f32> = value()?
                    .split(',')
                    .map(|p| p.trim().parse().map_err(|e| format!("{e}")))
                    .collect::<Result<_, _>>()?;
                args.camera = parts
                    .try_into()
                    .map_err(|_| "--camera braucht 5 Zahlen".to_string())?;
            }
            _ => return Err(format!("unbekanntes Argument {arg}")),
        }
    }
    Ok(args)
}

/// Sent by the wake thread when the Wayland socket or a client has data.
struct Wake;

/// Polls the compositor fds on a helper thread and wakes the winit loop.
/// After each wake it waits for an acknowledgement, so it only polls again
/// once the main thread has dispatched (otherwise the fds stay readable).
fn spawn_wake_thread(fds: Vec<OwnedFd>, proxy: EventLoopProxy<Wake>) -> mpsc::Sender<()> {
    let (ack_tx, ack_rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("wayland-wake".into())
        .spawn(move || {
            loop {
                let mut pfds: Vec<PollFd> = fds
                    .iter()
                    .map(|fd| PollFd::new(fd, PollFlags::IN))
                    .collect();
                match rustix::event::poll(&mut pfds, None) {
                    Ok(_) => {}
                    Err(rustix::io::Errno::INTR) => continue,
                    Err(e) => {
                        tracing::error!("poll fehlgeschlagen: {e}");
                        return;
                    }
                }
                if proxy.send_event(Wake).is_err() || ack_rx.recv().is_err() {
                    return;
                }
            }
        })
        .expect("Weck-Thread starten");
    ack_tx
}

struct App {
    args: Args,
    shell: Shell,
    wake_ack: mpsc::Sender<()>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: Camera,
    pressed: HashSet<KeyCode>,
    grabbed: bool,
    last_frame: Instant,
    screenshot_at: Option<Instant>,
    children: Vec<Child>,
    /// Frames drawn since `stats_since`, logged every 10 s.
    frames: u32,
    stats_since: Instant,
    /// Last cursor position in physical window pixels.
    cursor: Option<(f64, f64)>,
    /// Screen whose window gets the keyboard; `None` = keys move the camera.
    keyboard_slot: Option<usize>,
    ctrl: bool,
    /// When clients last got their frame callbacks.
    last_callbacks: Instant,
    script: VecDeque<Step>,
    /// When the next script step runs; `None` = no script running.
    script_at: Option<Instant>,
}

/// GNOME stops redraws of a hidden window; clients then still get frame
/// callbacks at this interval so they keep running.
const FALLBACK_FRAME: Duration = Duration::from_millis(50);

impl App {
    fn spawn_clients(&mut self) {
        for cmd in &self.args.spawn {
            let mut parts = cmd.split_whitespace();
            let Some(program) = parts.next() else {
                continue;
            };
            let mut command = Command::new(program);
            command
                .args(parts)
                .env("WAYLAND_DISPLAY", &self.shell.socket_name)
                .env_remove("DISPLAY");
            // SAFETY: prctl is async-signal-safe; children die with the compositor.
            unsafe {
                command.pre_exec(|| {
                    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                    Ok(())
                });
            }
            match command.spawn() {
                Ok(child) => self.children.push(child),
                Err(e) => tracing::error!("kann {cmd} nicht starten: {e}"),
            }
        }
    }

    /// Uploads changed client buffers into their screen textures.
    fn apply_shell_changes(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        let changed = self.shell.state.take_changed();
        if changed.is_empty() {
            return;
        }
        let popup_places = self.shell.popup_places();
        for target in changed {
            let layout = self.shell.layout(&target);
            let shell = &mut self.shell;
            let content = |s: &compositor::SurfaceLayout,
                           f: &mut dyn FnMut(compositor::ScreenContent)| {
                shell.with_surface_content(&s.surface, f)
            };
            match (target, layout) {
                (Target::Screen(slot), _) if slot >= renderer.screen_count() => {}
                (Target::Screen(slot), Some(layout)) => {
                    renderer.update_window(slot, layout.size, &layout.surfaces, content);
                }
                (Target::Screen(slot), None) => renderer.reset_screen(slot),
                (Target::Popup(id), Some(layout)) => {
                    match popup_places.iter().find(|(pid, ..)| pid == &id) {
                        Some((_, slot, pos, depth)) => renderer.update_popup(
                            id,
                            *slot,
                            *pos,
                            *depth,
                            layout.size,
                            &layout.surfaces,
                            content,
                        ),
                        None => renderer.remove_popup(&id),
                    }
                }
                (Target::Popup(id), None) => renderer.remove_popup(&id),
            }
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Screen and window position under the cursor.
    fn hit(&self) -> Option<(Target, (f64, f64))> {
        let renderer = self.renderer.as_ref()?;
        let (target, uv) = renderer.pick(&self.camera, self.cursor?)?;
        let size = self.shell.layout(&target)?.size;
        Some((
            target,
            (
                f64::from(uv.x) * f64::from(size.0),
                f64::from(uv.y) * f64::from(size.1),
            ),
        ))
    }

    fn update_pointer(&mut self) {
        let hit = if self.grabbed { None } else { self.hit() };
        if hit.is_none()
            && let Some(window) = &self.window
        {
            window.set_cursor(CursorIcon::Default);
        }
        self.shell.pointer_motion(hit);
    }

    fn apply_cursor(&mut self) {
        let (Some(status), Some(window)) = (self.shell.take_cursor(), &self.window) else {
            return;
        };
        if self.grabbed {
            return;
        }
        match status {
            CursorImageStatus::Hidden => window.set_cursor_visible(false),
            CursorImageStatus::Named(icon) => {
                window.set_cursor_visible(true);
                window.set_cursor(icon);
            }
            // Client-drawn cursor images are not shown yet; keep the host arrow.
            CursorImageStatus::Surface(_) => {
                window.set_cursor_visible(true);
                window.set_cursor(CursorIcon::Default);
            }
        }
    }

    fn set_keyboard_slot(&mut self, slot: Option<usize>) {
        if self.keyboard_slot != slot {
            self.keyboard_slot = slot;
            self.pressed.clear();
            self.shell.focus_keyboard(slot);
            match slot {
                Some(slot) => tracing::info!("Tastatur an Screen {slot}"),
                None => tracing::info!("Tastatur an die Welt"),
            }
        }
    }

    fn on_mouse_input(&mut self, button: MouseButton, pressed: bool) {
        if self.grabbed {
            return;
        }
        let hit = self.hit();
        let on_popup = matches!(hit, Some((Target::Popup(_), _)));
        if pressed && self.shell.grab_active() && !on_popup {
            // Placeholder: a click outside an open menu only closes it.
            self.shell.dismiss_popups();
            return;
        }
        if pressed && button == MouseButton::Left && hit.is_none() {
            // Click into the world: look around.
            self.set_keyboard_slot(None);
            self.shell.pointer_motion(None);
            self.set_grab(true);
            return;
        }
        let slot = match &hit {
            Some((Target::Screen(slot), _)) => Some(*slot),
            Some((Target::Popup(id), _)) => self.shell.popup_slot(id),
            None => None,
        };
        if pressed && slot.is_some() && !on_popup {
            // Placeholder D10: keyboard focus by click.
            self.set_keyboard_slot(slot);
        }
        let code = match button {
            MouseButton::Left => 0x110,
            MouseButton::Right => 0x111,
            MouseButton::Middle => 0x112,
            MouseButton::Back => 0x116,
            MouseButton::Forward => 0x115,
            MouseButton::Other(_) => return,
        };
        self.shell.pointer_button(code, pressed);
    }

    fn on_wheel(&mut self, delta: MouseScrollDelta) {
        if self.grabbed {
            return;
        }
        match delta {
            // winit: positive = up/left; Wayland: positive = down/right.
            MouseScrollDelta::LineDelta(x, y) => self.shell.pointer_axis(
                (f64::from(-x) * 15.0, f64::from(-y) * 15.0),
                Some(((-x * 120.0) as i32, (-y * 120.0) as i32)),
            ),
            MouseScrollDelta::PixelDelta(p) => self.shell.pointer_axis((-p.x, -p.y), None),
        }
    }

    fn on_key(&mut self, key: PhysicalKey, pressed: bool, repeat: bool) {
        if self.keyboard_slot.is_some() && !self.grabbed {
            // Placeholder escape key: Ctrl+Escape gives the keyboard back to the world.
            if key == PhysicalKey::Code(KeyCode::Escape) && self.ctrl {
                if pressed {
                    self.set_keyboard_slot(None);
                }
                return;
            }
            if repeat {
                // Clients repeat keys themselves.
                return;
            }
            if let Some(scancode) = key.to_scancode() {
                self.shell.key(scancode, pressed);
            }
            return;
        }
        let PhysicalKey::Code(code) = key else {
            return;
        };
        if pressed {
            match code {
                KeyCode::Escape => self.set_grab(false),
                KeyCode::F12 if !repeat => {
                    let secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    self.save_screenshot(format!("screenshot-{secs}.png").into());
                }
                _ => {}
            }
            if self.pressed.is_empty() {
                self.last_frame = Instant::now();
            }
            self.pressed.insert(code);
        } else {
            self.pressed.remove(&code);
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    /// Runs script steps that are due.
    fn run_script(&mut self, event_loop: &ActiveEventLoop) {
        while let Some(at) = self.script_at
            && Instant::now() >= at
        {
            let Some(step) = self.script.pop_front() else {
                self.script_at = None;
                tracing::info!("Skript beendet");
                return;
            };
            tracing::info!("Skript: {step:?}");
            match step {
                Step::Wait(d) => self.script_at = Some(Instant::now() + d),
                Step::Point { slot, x, y } => {
                    let (Some(renderer), Some(layout)) =
                        (&self.renderer, self.shell.layout(&Target::Screen(slot)))
                    else {
                        tracing::warn!("Skript: Screen {slot} zeigt kein Fenster");
                        continue;
                    };
                    let uv = Vec2::new(
                        ((x + 0.5) / f64::from(layout.size.0)) as f32,
                        ((y + 0.5) / f64::from(layout.size.1)) as f32,
                    );
                    self.cursor = Some(renderer.screen_point(&self.camera, slot, uv));
                    self.update_pointer();
                    // Round trip through the raycast: how far from the target pixel?
                    match self.hit() {
                        Some((Target::Screen(hit_slot), (hx, hy))) if hit_slot == slot => {
                            tracing::info!(
                                "Zeiger: Ziel ({x}, {y}), getroffen ({:.2}, {:.2}), Abweichung {:.2} px",
                                hx - 0.5,
                                hy - 0.5,
                                ((hx - 0.5 - x).powi(2) + (hy - 0.5 - y).powi(2)).sqrt()
                            )
                        }
                        other => tracing::warn!("Zeiger: Ziel verfehlt, getroffen {other:?}"),
                    }
                }
                Step::Cursor { x, y } => {
                    self.cursor = Some((x, y));
                    self.update_pointer();
                }
                Step::Click(button) => {
                    self.on_mouse_input(button, true);
                    self.on_mouse_input(button, false);
                }
                Step::Press(button) => self.on_mouse_input(button, true),
                Step::Release(button) => self.on_mouse_input(button, false),
                Step::Scroll(notches) => self.on_wheel(MouseScrollDelta::LineDelta(0.0, -notches)),
                Step::Keys(keys) => {
                    for &key in &keys {
                        if key == KeyCode::ControlLeft {
                            self.ctrl = true;
                        }
                        self.on_key(PhysicalKey::Code(key), true, false);
                    }
                    for &key in keys.iter().rev() {
                        self.on_key(PhysicalKey::Code(key), false, false);
                        if key == KeyCode::ControlLeft {
                            self.ctrl = false;
                        }
                    }
                }
                Step::Screenshot(path) => self.save_screenshot(path.into()),
                Step::WindowShot { slot, path } => {
                    if let Some(renderer) = &mut self.renderer {
                        match renderer.window_screenshot(slot, Path::new(&path)) {
                            Ok(()) => tracing::info!("Fensterbild gespeichert: {path}"),
                            Err(e) => tracing::error!("Fensterbild fehlgeschlagen: {e}"),
                        }
                    }
                }
                Step::Quit => {
                    event_loop.exit();
                    self.script_at = None;
                }
            }
        }
    }

    fn movement(&self) -> Vec3 {
        let axis = |pos: KeyCode, neg: KeyCode| {
            (self.pressed.contains(&pos) as i32 - self.pressed.contains(&neg) as i32) as f32
        };
        Vec3::new(
            axis(KeyCode::KeyD, KeyCode::KeyA),
            axis(KeyCode::Space, KeyCode::ShiftLeft),
            axis(KeyCode::KeyW, KeyCode::KeyS),
        )
    }

    fn set_grab(&mut self, grab: bool) {
        let Some(window) = &self.window else {
            return;
        };
        if grab {
            let result = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            if let Err(e) = result {
                tracing::warn!("Maus kann nicht gefangen werden: {e}");
                return;
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
        }
        window.set_cursor_visible(!grab);
        self.grabbed = grab;
        if !grab {
            self.update_pointer();
        }
    }

    fn save_screenshot(&mut self, path: PathBuf) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match renderer.screenshot(&self.camera, &path) {
            Ok(()) => tracing::info!("Screenshot gespeichert: {}", path.display()),
            Err(e) => tracing::error!("Screenshot fehlgeschlagen: {e}"),
        }
    }
}

impl ApplicationHandler<Wake> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Minecraft Shell – nested")
            .with_inner_size(winit::dpi::LogicalSize::new(1600.0, 900.0));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Fenster erstellen"),
        );
        let renderer = pollster::block_on(Renderer::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.shell.enable_gpu(renderer.device());
        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.screenshot_at = self
            .args
            .screenshot
            .as_ref()
            .map(|_| Instant::now() + self.args.screenshot_delay);
        self.spawn_clients();
        if !self.script.is_empty() {
            self.script_at = Some(Instant::now());
        }
        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: Wake) {
        if let Err(e) = self.shell.dispatch() {
            tracing::error!("Wayland-Dispatch fehlgeschlagen: {e}");
        }
        self.apply_shell_changes();
        self.apply_cursor();
        let _ = self.wake_ack.send(());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // While a script runs, real input would disturb the test.
        let real_input = matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::Focused(_)
        );
        if real_input && self.script_at.is_some() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::Focused(false) => {
                self.pressed.clear();
                self.set_grab(false);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.ctrl = modifiers.state().control_key();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some((position.x, position.y));
                if !self.grabbed {
                    self.update_pointer();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor = None;
                self.shell.pointer_motion(None);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_input(button, state == ElementState::Pressed);
            }
            WindowEvent::MouseWheel { delta, .. } => self.on_wheel(delta),
            WindowEvent::KeyboardInput { event, .. } => {
                self.on_key(
                    event.physical_key,
                    event.state == ElementState::Pressed,
                    event.repeat,
                );
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.1);
                self.last_frame = now;
                let movement = self.movement();
                let fast = self.pressed.contains(&KeyCode::ControlLeft);
                let speed = if fast { 12.0 } else { 4.0 };
                self.camera.translate(movement, speed * dt);
                if movement != Vec3::ZERO && !self.grabbed {
                    self.update_pointer();
                }

                if let Some(renderer) = &mut self.renderer {
                    renderer.render(&self.camera, self.shell.gpu_buffers());
                }
                self.shell.send_frame_callbacks();
                self.last_callbacks = Instant::now();
                self.frames += 1;
                let elapsed = self.stats_since.elapsed();
                if elapsed >= Duration::from_secs(10) {
                    tracing::info!(
                        "{} Bilder in {:.0} s ({:.1}/s)",
                        self.frames,
                        elapsed.as_secs_f64(),
                        f64::from(self.frames) / elapsed.as_secs_f64()
                    );
                    self.frames = 0;
                    self.stats_since = Instant::now();
                }
                if movement != Vec3::ZERO
                    && let Some(window) = &self.window
                {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event
            && self.grabbed
            && self.script_at.is_none()
        {
            self.camera.rotate(delta.0, delta.1);
            // The window under the crosshair changes while looking around;
            // the pointer only targets windows again once the mouse is free.
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if let Some(at) = self.screenshot_at
            && now >= at
        {
            self.screenshot_at = None;
            if let Some(path) = self.args.screenshot.clone() {
                self.save_screenshot(path);
            }
            if self.args.exit_after_screenshot {
                event_loop.exit();
            }
        }

        self.run_script(event_loop);

        let mut deadline = self.screenshot_at;
        if let Some(at) = self.script_at {
            deadline = Some(deadline.map_or(at, |d| d.min(at)));
        }
        if self.shell.wants_frame() {
            let fallback = self.last_callbacks + FALLBACK_FRAME;
            if now >= fallback {
                // No redraw from the host for a while (window hidden).
                self.shell.send_frame_callbacks();
                self.last_callbacks = now;
            } else {
                deadline = Some(deadline.map_or(fallback, |d| d.min(fallback)));
            }
        }
        event_loop.set_control_flow(match deadline {
            Some(at) => ControlFlow::WaitUntil(at),
            None => ControlFlow::Wait,
        });
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// First keyboard layout configured in GNOME, e.g. `("de", "nodeadkeys")`.
fn host_keyboard_layout() -> (String, String) {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.input-sources", "sources"])
        .output();
    let text = output
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    // Format: [('xkb', 'de+nodeadkeys'), ...]
    let first = text
        .split("('xkb', '")
        .nth(1)
        .and_then(|rest| rest.split('\'').next())
        .unwrap_or("us");
    let (layout, variant) = first.split_once('+').unwrap_or((first, ""));
    (layout.to_string(), variant.to_string())
}

fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,wgpu_core=warn,wgpu_hal=warn,naga=warn".into());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    let event_loop = EventLoop::<Wake>::with_user_event()
        .build()
        .expect("Ereignisschleife erstellen");
    let (layout, variant) = host_keyboard_layout();
    tracing::info!("Tastaturlayout: {layout} {variant}");
    let shell = Shell::new(2, &layout, &variant).expect("Wayland-Server starten");
    let qwertz = ["de", "at", "ch", "cz", "sk", "hu", "si", "hr"].contains(&layout.as_str());
    let script = match &args.script {
        Some(path) => match std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|text| script::parse(&text, qwertz))
        {
            Ok(steps) => steps.into(),
            Err(e) => {
                eprintln!("Skript {}: {e}", path.display());
                std::process::exit(2);
            }
        },
        None => VecDeque::new(),
    };
    tracing::info!(
        "Wayland-Socket: {} (Programme starten mit WAYLAND_DISPLAY={})",
        shell.socket_name.to_string_lossy(),
        shell.socket_name.to_string_lossy()
    );
    let wake_ack = spawn_wake_thread(
        vec![shell.wake_fd().expect("fd duplizieren")],
        event_loop.create_proxy(),
    );

    let [x, y, z, yaw, pitch] = args.camera;
    let camera = Camera::new(Vec3::new(x, y, z), yaw.to_radians(), pitch.to_radians());
    let mut app = App {
        args,
        shell,
        wake_ack,
        window: None,
        renderer: None,
        camera,
        pressed: HashSet::new(),
        grabbed: false,
        last_frame: Instant::now(),
        screenshot_at: None,
        children: Vec::new(),
        frames: 0,
        stats_since: Instant::now(),
        cursor: None,
        keyboard_slot: None,
        ctrl: false,
        last_callbacks: Instant::now(),
        script,
        script_at: None,
    };
    event_loop.run_app(&mut app).expect("Ereignisschleife");
}
