//! M0 prototype: nested Wayland compositor that shows client windows as
//! textures on tilted quads in a free-flying 3D scene.
//!
//! Controls: left click captures the mouse, Escape releases it, WASD moves,
//! Space/Shift go up/down, Ctrl moves faster, F12 saves a screenshot.

mod camera;
mod compositor;
mod render;

use std::collections::HashSet;
use std::os::fd::OwnedFd;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use glam::Vec3;
use rustix::event::{PollFd, PollFlags};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use camera::Camera;
use compositor::Shell;
use render::Renderer;

const USAGE: &str = "\
nested [--spawn BEFEHL]... [--screenshot DATEI] [--screenshot-delay SEK] [--exit-after-screenshot]
       [--camera X,Y,Z,GIER,NICK]

  --spawn BEFEHL            BEFEHL (an Leerzeichen getrennt) als Programm in diesem Compositor starten
  --screenshot DATEI        PNG der Szene nach der Wartezeit speichern
  --screenshot-delay SEK    Wartezeit für --screenshot (Standard 3)
  --exit-after-screenshot   nach dem Screenshot beenden
  --camera X,Y,Z,GIER,NICK  Startposition, Winkel in Grad (Standard 0,1.6,3.2,0,-3)

Steuerung: Linksklick fängt die Maus, Esc gibt sie frei, WASD bewegt,
Leertaste/Shift hoch/runter, Strg schneller, F12 speichert einen Screenshot.";

struct Args {
    spawn: Vec<String>,
    screenshot: Option<PathBuf>,
    screenshot_delay: Duration,
    exit_after_screenshot: bool,
    camera: [f32; 5],
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        spawn: Vec::new(),
        screenshot: None,
        screenshot_delay: Duration::from_secs(3),
        exit_after_screenshot: false,
        camera: [0.0, 1.6, 3.2, 0.0, -3.0],
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
}

impl App {
    fn spawn_clients(&mut self) {
        for cmd in &self.args.spawn {
            let mut parts = cmd.split_whitespace();
            let Some(program) = parts.next() else {
                continue;
            };
            match Command::new(program)
                .args(parts)
                .env("WAYLAND_DISPLAY", &self.shell.socket_name)
                .env_remove("DISPLAY")
                .spawn()
            {
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
        for slot in changed {
            if slot >= renderer.screen_count() {
                continue;
            }
            self.shell.with_screen_image(slot, |image| match image {
                Some(image) => renderer.set_screen_image(slot, image),
                None => renderer.reset_screen(slot),
            });
        }
        if let Some(window) = &self.window {
            window.request_redraw();
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
        self.window = Some(window.clone());
        self.renderer = Some(renderer);
        self.screenshot_at = self
            .args
            .screenshot
            .as_ref()
            .map(|_| Instant::now() + self.args.screenshot_delay);
        self.spawn_clients();
        window.request_redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: Wake) {
        if let Err(e) = self.shell.dispatch() {
            tracing::error!("Wayland-Dispatch fehlgeschlagen: {e}");
        }
        self.apply_shell_changes();
        let _ = self.wake_ack.send(());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } if !self.grabbed => self.set_grab(true),
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                if event.state == ElementState::Pressed {
                    match code {
                        KeyCode::Escape => self.set_grab(false),
                        KeyCode::F12 if !event.repeat => {
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
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.1);
                self.last_frame = now;
                let movement = self.movement();
                let fast = self.pressed.contains(&KeyCode::ControlLeft);
                let speed = if fast { 12.0 } else { 4.0 };
                self.camera.translate(movement, speed * dt);

                if let Some(renderer) = &mut self.renderer {
                    renderer.render(&self.camera);
                }
                self.shell.send_frame_callbacks();
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
        {
            self.camera.rotate(delta.0, delta.1);
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        match self.screenshot_at {
            Some(at) if Instant::now() >= at => {
                self.screenshot_at = None;
                if let Some(path) = self.args.screenshot.clone() {
                    self.save_screenshot(path);
                }
                if self.args.exit_after_screenshot {
                    event_loop.exit();
                }
                event_loop.set_control_flow(ControlFlow::Wait);
            }
            Some(at) => event_loop.set_control_flow(ControlFlow::WaitUntil(at)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
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
    let mut shell = Shell::new(2).expect("Wayland-Server starten");
    tracing::info!(
        "Wayland-Socket: {} (Programme starten mit WAYLAND_DISPLAY={})",
        shell.socket_name.to_string_lossy(),
        shell.socket_name.to_string_lossy()
    );
    let wake_ack = spawn_wake_thread(
        shell.wake_fds().expect("fds duplizieren"),
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
    };
    event_loop.run_app(&mut app).expect("Ereignisschleife");
}
