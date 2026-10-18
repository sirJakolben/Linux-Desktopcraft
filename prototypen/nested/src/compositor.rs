use std::collections::BTreeSet;
use std::ffi::OsString;
use std::os::fd::{AsFd, OwnedFd};
use std::sync::Arc;
use std::time::Instant;

use smithay::backend::renderer::utils::{on_commit_buffer_handler, with_renderer_surface_state};
use smithay::input::keyboard::XkbConfig;
use smithay::input::pointer::CursorImageStatus;
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Mode, Output, PhysicalProperties, Scale, Subpixel};
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, Display, ListeningSocket};
use smithay::utils::{Serial, Transform};
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{
    CompositorClientState, CompositorHandler, CompositorState, SurfaceAttributes, TraversalAction,
    get_parent, with_surface_tree_downward,
};
use smithay::wayland::output::{OutputHandler, OutputManagerState};
use smithay::wayland::pointer_constraints::PointerConstraintsHandler;
use smithay::wayland::selection::SelectionHandler;
use smithay::wayland::selection::data_device::{
    DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler,
};
use smithay::wayland::shell::xdg::decoration::{XdgDecorationHandler, XdgDecorationState};
use smithay::wayland::shell::xdg::{
    PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
};
use smithay::wayland::shm::{self, ShmHandler, ShmState};

use crate::render::Image;

/// Logical size suggested to new toplevels (16:10, like the screens).
const WINDOW_SIZE: (i32, i32) = (1024, 640);

/// The Wayland server side: socket, globals, and the toplevel shown on each screen.
pub struct Shell {
    display: Display<ShellState>,
    socket: ListeningSocket,
    pub socket_name: OsString,
    pub state: ShellState,
}

pub struct ShellState {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    _xdg_decoration_state: XdgDecorationState,
    shm_state: ShmState,
    data_device_state: DataDeviceState,
    seat_state: SeatState<Self>,
    _output_manager_state: OutputManagerState,
    _seat: Seat<Self>,
    _output: Output,
    /// Toplevel shown on each screen quad; placeholder for D9: the next free one.
    screens: Vec<Option<ToplevelSurface>>,
    /// Screens whose content changed since the last `take_changed`.
    changed: BTreeSet<usize>,
    start: Instant,
}

impl Shell {
    pub fn new(screen_count: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let display = Display::<ShellState>::new()?;
        let dh = display.handle();

        let compositor_state = CompositorState::new::<ShellState>(&dh);
        let xdg_shell_state = XdgShellState::new::<ShellState>(&dh);
        let xdg_decoration_state = XdgDecorationState::new::<ShellState>(&dh);
        let shm_state = ShmState::new::<ShellState>(
            &dh,
            vec![wl_shm::Format::Abgr8888, wl_shm::Format::Xbgr8888],
        );
        let data_device_state = DataDeviceState::new::<ShellState>(&dh);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<ShellState>(&dh);

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "seat0");
        seat.add_keyboard(XkbConfig::default(), 200, 25)?;
        seat.add_pointer();

        let output = Output::new(
            "nested-1".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "Minecraft Shell".into(),
                model: "nested".into(),
                serial_number: "0".into(),
            },
        );
        let _ = output.create_global::<ShellState>(&dh);
        let mode = Mode {
            size: WINDOW_SIZE.into(),
            refresh: 60_000,
        };
        output.change_current_state(
            Some(mode),
            Some(Transform::Normal),
            Some(Scale::Integer(1)),
            Some((0, 0).into()),
        );
        output.set_preferred(mode);

        let socket = ListeningSocket::bind_auto("wayland", 1..33)?;
        let socket_name = socket
            .socket_name()
            .map(|n| n.to_os_string())
            .ok_or("Socket ohne Namen")?;

        Ok(Self {
            display,
            socket,
            socket_name,
            state: ShellState {
                compositor_state,
                xdg_shell_state,
                _xdg_decoration_state: xdg_decoration_state,
                shm_state,
                data_device_state,
                seat_state,
                _output_manager_state: output_manager_state,
                _seat: seat,
                _output: output,
                screens: vec![None; screen_count],
                changed: BTreeSet::new(),
                start: Instant::now(),
            },
        })
    }

    /// File descriptors that become readable when `dispatch` has work.
    pub fn wake_fds(&mut self) -> std::io::Result<Vec<OwnedFd>> {
        Ok(vec![
            self.display.backend().poll_fd().try_clone_to_owned()?,
            self.socket.as_fd().try_clone_to_owned()?,
        ])
    }

    /// Accepts new clients and handles all pending requests.
    pub fn dispatch(&mut self) -> std::io::Result<()> {
        while let Some(stream) = self.socket.accept()? {
            let client = self
                .display
                .handle()
                .insert_client(stream, Arc::new(ClientState::default()))?;
            tracing::info!("Programm verbunden: {:?}", client.id());
        }
        self.display.dispatch_clients(&mut self.state)?;
        self.display.flush_clients()?;
        Ok(())
    }

    pub fn flush(&mut self) {
        if let Err(e) = self.display.flush_clients() {
            tracing::warn!("flush fehlgeschlagen: {e}");
        }
    }

    /// Calls `f` with the current shm contents of screen `slot`, or `None` if
    /// the screen has no client buffer (unmapped, no window, or not shm).
    pub fn with_screen_image(&self, slot: usize, f: impl FnOnce(Option<&Image>)) {
        let Some(toplevel) = &self.state.screens[slot] else {
            return f(None);
        };
        let buffer =
            with_renderer_surface_state(toplevel.wl_surface(), |s| s.buffer().cloned()).flatten();
        let Some(buffer) = buffer else {
            return f(None);
        };
        let mut f = Some(f);
        let result = shm::with_buffer_contents(&buffer, |ptr, len, data| {
            let Some(format) = shm_format(data.format) else {
                tracing::warn!("shm-Format nicht unterstützt: {:?}", data.format);
                return false;
            };
            let (offset, stride) = (data.offset as usize, data.stride as usize);
            let (width, height) = (data.width as u32, data.height as u32);
            if width == 0 || height == 0 {
                return false;
            }
            let needed = stride * (height as usize - 1) + width as usize * 4;
            if offset + needed > len {
                tracing::warn!("shm-Puffer außerhalb des Pools");
                return false;
            }
            // SAFETY: smithay guarantees `ptr..ptr+len` is the mapped pool for
            // the duration of this closure; bounds were checked above.
            let pool = unsafe { std::slice::from_raw_parts(ptr, len) };
            let image = Image {
                width,
                height,
                stride: stride as u32,
                format,
                data: &pool[offset..offset + needed],
            };
            (f.take().unwrap())(Some(&image));
            true
        });
        match result {
            Ok(true) => {}
            Ok(false) => (f.take().unwrap())(None),
            Err(e) => {
                // dmabuf and other buffer types come with A3.
                tracing::warn!("Puffer von Screen {slot} nicht lesbar: {e:?}");
                (f.take().unwrap())(None);
            }
        }
    }

    /// Answers frame callbacks of every shown surface tree, after a frame was drawn.
    pub fn send_frame_callbacks(&mut self) {
        let time = self.state.start.elapsed().as_millis() as u32;
        for toplevel in self.state.screens.iter().flatten() {
            with_surface_tree_downward(
                toplevel.wl_surface(),
                (),
                |_, _, &()| TraversalAction::DoChildren(()),
                |_, states, &()| {
                    for callback in states
                        .cached_state
                        .get::<SurfaceAttributes>()
                        .current()
                        .frame_callbacks
                        .drain(..)
                    {
                        callback.done(time);
                    }
                },
                |_, _, &()| true,
            );
        }
        self.flush();
    }
}

impl ShellState {
    pub fn take_changed(&mut self) -> BTreeSet<usize> {
        std::mem::take(&mut self.changed)
    }

    fn screen_of(&self, surface: &WlSurface) -> Option<usize> {
        self.screens
            .iter()
            .position(|t| t.as_ref().is_some_and(|t| t.wl_surface() == surface))
    }
}

/// Maps a little-endian wl_shm format to the wgpu format with the same memory layout.
fn shm_format(format: wl_shm::Format) -> Option<wgpu::TextureFormat> {
    match format {
        // ARGB8888 is stored as B, G, R, A bytes in memory.
        wl_shm::Format::Argb8888 | wl_shm::Format::Xrgb8888 => {
            Some(wgpu::TextureFormat::Bgra8UnormSrgb)
        }
        // ABGR8888 is stored as R, G, B, A bytes in memory.
        wl_shm::Format::Abgr8888 | wl_shm::Format::Xbgr8888 => {
            Some(wgpu::TextureFormat::Rgba8UnormSrgb)
        }
        _ => None,
    }
}

impl CompositorHandler for ShellState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);

        let mut root = surface.clone();
        while let Some(parent) = get_parent(&root) {
            root = parent;
        }
        let Some(slot) = self.screen_of(&root) else {
            return;
        };
        let toplevel = self.screens[slot].as_ref().unwrap();
        if !toplevel.is_initial_configure_sent() {
            toplevel.send_configure();
        }
        self.changed.insert(slot);
    }
}

impl BufferHandler for ShellState {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for ShellState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl XdgShellHandler for ShellState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.size = Some(WINDOW_SIZE.into());
            state.states.set(xdg_toplevel::State::Activated);
            // Tiled on all sides: clients drop shadows and rounded corners.
            state.states.set(xdg_toplevel::State::TiledLeft);
            state.states.set(xdg_toplevel::State::TiledRight);
            state.states.set(xdg_toplevel::State::TiledTop);
            state.states.set(xdg_toplevel::State::TiledBottom);
        });
        match self.screens.iter().position(Option::is_none) {
            Some(slot) => {
                tracing::info!("neues Fenster auf Screen {slot}");
                self.screens[slot] = Some(surface);
            }
            None => tracing::warn!("neues Fenster, aber alle Screens belegt"),
        }
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        if let Some(slot) = self.screen_of(surface.wl_surface()) {
            tracing::info!("Fenster auf Screen {slot} geschlossen");
            self.screens[slot] = None;
            self.changed.insert(slot);
        }
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        // Pop-ups come with A5.
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {}

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
    }
}

impl XdgDecorationHandler for ShellState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(DecorationMode::ServerSide);
        });
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: DecorationMode) {
        // Always server side: no client title bars on the wall.
        self.new_decoration(toplevel.clone());
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        self.request_mode(toplevel, DecorationMode::ServerSide);
    }
}

impl SeatHandler for ShellState {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

impl OutputHandler for ShellState {}

impl PointerConstraintsHandler for ShellState {}

// Clipboard: foot refuses to start without wl_data_device_manager.
impl SelectionHandler for ShellState {
    type SelectionUserData = ();
}

impl DataDeviceHandler for ShellState {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}

impl WaylandDndGrabHandler for ShellState {}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, client_id: ClientId, reason: DisconnectReason) {
        tracing::info!("Programm {client_id:?} getrennt: {reason:?}");
    }
}

smithay::delegate_dispatch2!(ShellState);
