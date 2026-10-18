use std::collections::HashSet;
use std::ffi::OsString;
use std::os::fd::{AsFd, OwnedFd};
use std::sync::Arc;
use std::time::{Duration, Instant};

use smithay::backend::allocator::{Buffer as _, Fourcc};
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::drm::{DrmDeviceFd, DrmNode, NodeType};
use smithay::backend::renderer::utils::{
    Buffer, CommitCounter, RendererSurfaceStateUserData, on_commit_buffer_handler, with_renderer_surface_state,
};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{EventLoop, Interest, LoopHandle, Mode as LoopMode, PostAction};
use smithay::reexports::wayland_server::{DisplayHandle, Resource};
use smithay::utils::DeviceFd;
use smithay::wayland::compositor::{
    BufferAssignment, add_blocker, add_pre_commit_hook, with_states,
    with_surface_tree_upward,
};
use smithay::wayland::shell::xdg::SurfaceCachedState;
use smithay::wayland::viewporter::ViewporterState;
use smithay::wayland::dmabuf::{
    DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier, get_dmabuf,
};
use smithay::wayland::drm_syncobj::{
    DrmSyncobjCachedState, DrmSyncobjHandler, DrmSyncobjState, supports_syncobj_eventfd,
};
use smithay::wayland::socket::ListeningSocketSource;
use smithay::backend::input::{Axis, AxisSource, ButtonState, InputTime, KeyState};
use smithay::input::keyboard::{FilterResult, Keycode, XkbConfig};
use smithay::input::pointer::{AxisFrame, ButtonEvent, MotionEvent};
use smithay::utils::{Logical, Point, Rectangle, SERIAL_COUNTER};
use smithay::input::pointer::CursorImageStatus;
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::output::{Mode, Output, PhysicalProperties, Scale, Subpixel};
use smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::backend::{
    ClientData, ClientId, DisconnectReason, ObjectId,
};
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Client, Display};
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

use crate::dmabuf::DmabufImporter;
use crate::render::Image;

/// Logical size suggested to new toplevels (16:10, like the screens).
const WINDOW_SIZE: (i32, i32) = (1024, 640);

/// The Wayland server side: socket, globals, and the toplevel shown on each screen.
pub struct Shell {
    event_loop: EventLoop<'static, ShellState>,
    pub socket_name: OsString,
    pub state: ShellState,
}

/// What a screen currently shows.
pub enum ScreenContent<'a> {
    /// No window or no buffer: show the test pattern.
    Empty,
    /// CPU memory, copied into a texture.
    Shm(&'a Image<'a>),
    /// GPU buffer imported without copy.
    Gpu {
        texture: wgpu::Texture,
        y_inverted: bool,
        /// Alpha channel is padding (XRGB-style formats).
        opaque: bool,
    },
}

/// Something with its own texture and quad: a toplevel window or a pop-up.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Screen(usize),
    Popup(ObjectId),
}

struct PopupEntry {
    surface: PopupSurface,
    /// Screen whose window the pop-up belongs to.
    slot: usize,
    /// 1 for pop-ups of the toplevel, +1 per nesting level.
    depth: u32,
    /// Position and size relative to the parent's window geometry.
    geometry: Rectangle<i32, Logical>,
    grab: bool,
}

/// Room around the window into which pop-ups may extend (placeholder; m0 "Pop-ups").
const POPUP_MARGIN: i32 = 32;

/// Where one surface of a window tree is drawn inside the window texture.
pub struct SurfaceLayout {
    pub surface: WlSurface,
    pub id: ObjectId,
    pub commit: CommitCounter,
    /// Logical pixels in the window: x, y, width, height.
    pub dst: [f32; 4],
    /// Part of the buffer shown: left, top, right, bottom in UV.
    pub uv: [f32; 4],
}

/// The surfaces of one window, bottom first, and the window size.
pub struct WindowLayout {
    pub size: (u32, u32),
    pub surfaces: Vec<SurfaceLayout>,
}

pub struct ShellState {
    display_handle: DisplayHandle,
    loop_handle: LoopHandle<'static, ShellState>,
    dmabuf_state: DmabufState,
    _dmabuf_global: Option<DmabufGlobal>,
    syncobj_state: Option<DrmSyncobjState>,
    importer: Option<DmabufImporter>,
    /// Surfaces whose sync path was already logged.
    sync_logged: HashSet<ObjectId>,
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    _xdg_decoration_state: XdgDecorationState,
    shm_state: ShmState,
    data_device_state: DataDeviceState,
    seat_state: SeatState<Self>,
    _output_manager_state: OutputManagerState,
    seat: Seat<Self>,
    /// Cursor requested by the client under the pointer, not yet applied.
    cursor: Option<CursorImageStatus>,
    _output: Output,
    /// Toplevel shown on each screen quad; placeholder for D9: the next free one.
    screens: Vec<Option<ToplevelSurface>>,
    /// Pop-ups of shown windows, in creation order.
    popups: Vec<PopupEntry>,
    /// Targets whose content changed since the last `take_changed`.
    changed: HashSet<Target>,
    start: Instant,
}

impl Shell {
    /// `xkb_layout` and `xkb_variant` should match the host keyboard.
    pub fn new(
        screen_count: usize,
        xkb_layout: &str,
        xkb_variant: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display = Display::<ShellState>::new()?;
        let dh = display.handle();
        let event_loop = EventLoop::<ShellState>::try_new()?;
        let loop_handle = event_loop.handle();

        let compositor_state = CompositorState::new::<ShellState>(&dh);
        let xdg_shell_state = XdgShellState::new::<ShellState>(&dh);
        let xdg_decoration_state = XdgDecorationState::new::<ShellState>(&dh);
        let shm_state = ShmState::new::<ShellState>(
            &dh,
            vec![wl_shm::Format::Abgr8888, wl_shm::Format::Xbgr8888],
        );
        let data_device_state = DataDeviceState::new::<ShellState>(&dh);
        ViewporterState::new::<ShellState>(&dh);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<ShellState>(&dh);

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "seat0");
        let xkb = XkbConfig {
            layout: xkb_layout,
            variant: xkb_variant,
            ..Default::default()
        };
        seat.add_keyboard(xkb, 600, 25)?;
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

        let socket = ListeningSocketSource::new_auto()?;
        let socket_name = socket.socket_name().to_os_string();
        loop_handle
            .insert_source(socket, |stream, _, state| {
                match state
                    .display_handle
                    .insert_client(stream, Arc::new(ClientState::default()))
                {
                    Ok(client) => tracing::info!("Programm verbunden: {:?}", client.id()),
                    Err(e) => tracing::warn!("Programm abgewiesen: {e}"),
                }
            })
            .map_err(|e| e.error)?;
        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, LoopMode::Level),
                |_, display, state| {
                    // SAFETY: the display is never dropped while the loop runs.
                    unsafe { display.get_mut().dispatch_clients(state)? };
                    Ok(PostAction::Continue)
                },
            )
            .map_err(|e| e.error)?;

        Ok(Self {
            event_loop,
            socket_name,
            state: ShellState {
                display_handle: dh.clone(),
                loop_handle,
                dmabuf_state: DmabufState::new(),
                _dmabuf_global: None,
                syncobj_state: None,
                importer: None,
                sync_logged: HashSet::new(),
                compositor_state,
                xdg_shell_state,
                _xdg_decoration_state: xdg_decoration_state,
                shm_state,
                data_device_state,
                seat_state,
                _output_manager_state: output_manager_state,
                seat,
                cursor: None,
                _output: output,
                screens: vec![None; screen_count],
                popups: Vec::new(),
                changed: HashSet::new(),
                start: Instant::now(),
            },
        })
    }

    /// File descriptor that becomes readable when `dispatch` has work.
    pub fn wake_fd(&self) -> std::io::Result<OwnedFd> {
        self.event_loop.as_fd().try_clone_to_owned()
    }

    /// Accepts new clients, handles pending requests and cleared sync blockers.
    pub fn dispatch(&mut self) -> std::io::Result<()> {
        self.event_loop
            .dispatch(Some(Duration::ZERO), &mut self.state)?;
        self.state.display_handle.flush_clients()?;
        Ok(())
    }

    pub fn flush(&mut self) {
        if let Err(e) = self.state.display_handle.flush_clients() {
            tracing::warn!("flush fehlgeschlagen: {e}");
        }
    }

    /// Offers linux-dmabuf (with feedback) and, if the kernel supports it,
    /// linux-drm-syncobj, both for the GPU `device` runs on.
    pub fn enable_gpu(&mut self, device: &wgpu::Device) {
        let importer = match DmabufImporter::new(device) {
            Ok(importer) => importer,
            Err(e) => {
                tracing::warn!("kein dmabuf-Import, nur shm: {e}");
                return;
            }
        };
        let node = DrmNode::from_dev_id(importer.render_node).ok();
        let path = node.and_then(|n| n.dev_path_with_type(NodeType::Render));
        tracing::info!(
            "dmabuf: Render-Node {:?}, {} Format/Modifier-Paare",
            path,
            importer.formats.len()
        );
        for format in &importer.formats {
            tracing::debug!(
                "  {:?} Modifier {:#x}",
                format.code,
                u64::from(format.modifier)
            );
        }
        let state = &mut self.state;
        match DmabufFeedbackBuilder::new(importer.render_node, importer.formats.clone()).build() {
            Ok(feedback) => {
                let global = state
                    .dmabuf_state
                    .create_global_with_default_feedback::<ShellState>(
                        &state.display_handle,
                        &feedback,
                    );
                state._dmabuf_global = Some(global);
            }
            Err(e) => tracing::warn!("dmabuf-Feedback fehlgeschlagen: {e}"),
        }

        let device_fd = path.and_then(|path| {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|e| tracing::warn!("{} nicht zu öffnen: {e}", path.display()))
                .ok()
        });
        if let Some(file) = device_fd {
            let fd = DrmDeviceFd::new(DeviceFd::from(OwnedFd::from(file)));
            if supports_syncobj_eventfd(&fd) {
                state.syncobj_state = Some(DrmSyncobjState::new::<ShellState>(
                    &state.display_handle,
                    fd,
                ));
                tracing::info!("Explicit Sync (linux-drm-syncobj-v1) angeboten");
            } else {
                tracing::warn!("Kernel ohne syncobj_eventfd: kein Explicit Sync");
            }
        }
        state.importer = Some(importer);
    }

    /// Buffers currently shown from GPU memory. The renderer keeps them alive
    /// until the GPU finished reading, so the release point fires only then.
    pub fn gpu_buffers(&self) -> Vec<Buffer> {
        let mut buffers = Vec::new();
        for root in self.state.roots() {
            with_surface_tree_downward(
                &root,
                (),
                |_, _, &()| TraversalAction::DoChildren(()),
                |_, states, &()| {
                    // Use `states` directly: the traversal already holds the surface lock.
                    let buffer = states
                        .data_map
                        .get::<RendererSurfaceStateUserData>()
                        .and_then(|d| d.lock().unwrap().buffer().cloned());
                    if let Some(buffer) = buffer
                        && get_dmabuf(&buffer).is_ok()
                    {
                        buffers.push(buffer);
                    }
                },
                |_, _, &()| true,
            );
        }
        buffers
    }

    /// Layout of `target`, or `None` if nothing is mapped.
    pub fn layout(&self, target: &Target) -> Option<WindowLayout> {
        self.state.layout(target)
    }

    /// Topmost surface of `target` at `pos` (target pixels) and that
    /// surface's origin in the same coordinates.
    fn surface_at(
        &self,
        target: &Target,
        pos: (f64, f64),
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        let layout = self.layout(target)?;
        layout
            .surfaces
            .iter()
            .rev()
            .find(|s| {
                let [x, y, w, h] = s.dst.map(f64::from);
                pos.0 >= x && pos.0 < x + w && pos.1 >= y && pos.1 < y + h
            })
            .map(|s| {
                let origin = (f64::from(s.dst[0]), f64::from(s.dst[1]));
                (s.surface.clone(), origin.into())
            })
    }

    fn input_time(&self) -> InputTime {
        InputTime::from_micros(self.state.start.elapsed().as_micros() as u64)
    }

    /// Pointer is over `target` at `pos` (target pixels), or over nothing.
    pub fn pointer_motion(&mut self, hit: Option<(Target, (f64, f64))>) {
        let pointer = self.state.seat.get_pointer().expect("seat has a pointer");
        let focus = hit
            .as_ref()
            .and_then(|(target, pos)| self.surface_at(target, *pos));
        let location = hit.as_ref().map(|(_, pos)| *pos).unwrap_or((-1.0, -1.0));
        let event = MotionEvent {
            location: location.into(),
            serial: SERIAL_COUNTER.next_serial(),
            time: self.input_time(),
        };
        pointer.motion(&mut self.state, focus, &event);
        pointer.frame(&mut self.state);
        self.flush();
    }

    /// `button` is a Linux input code such as `BTN_LEFT` (0x110).
    pub fn pointer_button(&mut self, button: u32, pressed: bool) {
        let pointer = self.state.seat.get_pointer().expect("seat has a pointer");
        let event = ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: self.input_time(),
            button,
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
        };
        pointer.button(&mut self.state, &event);
        pointer.frame(&mut self.state);
        self.flush();
    }

    /// Scrolls by `delta` logical pixels (positive = right/down). `steps` are
    /// wheel notches in 1/120 units, `None` for touchpads.
    pub fn pointer_axis(&mut self, delta: (f64, f64), steps: Option<(i32, i32)>) {
        let pointer = self.state.seat.get_pointer().expect("seat has a pointer");
        let mut frame = AxisFrame::new(self.input_time()).source(if steps.is_some() {
            AxisSource::Wheel
        } else {
            AxisSource::Finger
        });
        if delta.0 != 0.0 {
            frame = frame.value(Axis::Horizontal, delta.0);
        }
        if delta.1 != 0.0 {
            frame = frame.value(Axis::Vertical, delta.1);
        }
        if let Some((sx, sy)) = steps {
            if sx != 0 {
                frame = frame.v120(Axis::Horizontal, sx);
            }
            if sy != 0 {
                frame = frame.v120(Axis::Vertical, sy);
            }
        }
        pointer.axis(&mut self.state, frame);
        pointer.frame(&mut self.state);
        self.flush();
    }

    /// Gives the keyboard to the window on `slot`, or takes it away.
    pub fn focus_keyboard(&mut self, slot: Option<usize>) {
        let keyboard = self.state.seat.get_keyboard().expect("seat has a keyboard");
        let focus = slot
            .and_then(|slot| self.state.screens[slot].as_ref())
            .map(|t| t.wl_surface().clone());
        keyboard.set_focus(&mut self.state, focus, SERIAL_COUNTER.next_serial());
        self.flush();
    }

    /// `scancode` is the Linux evdev key code.
    pub fn key(&mut self, scancode: u32, pressed: bool) {
        let keyboard = self.state.seat.get_keyboard().expect("seat has a keyboard");
        let time = self.input_time();
        keyboard.input::<(), _>(
            &mut self.state,
            Keycode::new(scancode + 8),
            if pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            },
            SERIAL_COUNTER.next_serial(),
            time,
            |_, _, _| FilterResult::Forward,
        );
        self.flush();
    }

    /// Pop-ups to draw: id, screen, position in that window's pixels, depth.
    pub fn popup_places(&self) -> Vec<(ObjectId, usize, (i32, i32), u32)> {
        self.state
            .popups
            .iter()
            .filter_map(|p| {
                let (slot, pos, depth) = self.state.place_of(p.surface.wl_surface())?;
                Some((p.surface.wl_surface().id(), slot, (pos.x, pos.y), depth))
            })
            .collect()
    }

    /// Screen of the window that owns pop-up `id`.
    pub fn popup_slot(&self, id: &ObjectId) -> Option<usize> {
        self.state
            .popups
            .iter()
            .find(|p| &p.surface.wl_surface().id() == id)
            .map(|p| p.slot)
    }

    /// Whether a pop-up holds a grab (menus): clicks elsewhere close it.
    pub fn grab_active(&self) -> bool {
        self.state.popups.iter().any(|p| p.grab)
    }

    /// Closes all pop-ups, innermost first.
    pub fn dismiss_popups(&mut self) {
        let mut popups: Vec<_> = self
            .state
            .popups
            .iter()
            .map(|p| (p.depth, p.surface.clone()))
            .collect();
        popups.sort_by_key(|(depth, _)| std::cmp::Reverse(*depth));
        for (_, popup) in popups {
            popup.send_popup_done();
        }
        tracing::info!("Pop-ups geschlossen (Klick außerhalb)");
        self.flush();
    }

    /// Cursor the client under the pointer asked for since the last call.
    pub fn take_cursor(&mut self) -> Option<CursorImageStatus> {
        self.state.cursor.take()
    }

    /// Calls `f` with the current buffer contents of `surface`.
    pub fn with_surface_content(&mut self, surface: &WlSurface, f: impl FnOnce(ScreenContent)) {
        let buffer = with_renderer_surface_state(surface, |s| s.buffer().cloned()).flatten();
        let Some(buffer) = buffer else {
            return f(ScreenContent::Empty);
        };
        if let Ok(dmabuf) = get_dmabuf(&buffer) {
            let imported = match &mut self.state.importer {
                Some(importer) => importer.import(dmabuf),
                None => Err("kein Importer".into()),
            };
            return match imported {
                Ok(texture) => f(ScreenContent::Gpu {
                    texture,
                    y_inverted: dmabuf.y_inverted(),
                    opaque: matches!(dmabuf.format().code, Fourcc::Xrgb8888 | Fourcc::Xbgr8888),
                }),
                Err(e) => {
                    tracing::warn!("dmabuf nicht importierbar: {e}");
                    f(ScreenContent::Empty)
                }
            };
        }
        let mut f = Some(f);
        let result = shm::with_buffer_contents(&buffer, |ptr, len, data| {
            let opaque = matches!(
                data.format,
                wl_shm::Format::Xrgb8888 | wl_shm::Format::Xbgr8888
            );
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
                opaque,
                data: &pool[offset..offset + needed],
            };
            (f.take().unwrap())(ScreenContent::Shm(&image));
            true
        });
        match result {
            Ok(true) => {}
            Ok(false) => (f.take().unwrap())(ScreenContent::Empty),
            Err(e) => {
                tracing::warn!("Puffer nicht lesbar: {e:?}");
                (f.take().unwrap())(ScreenContent::Empty);
            }
        }
    }

    /// Whether any shown surface waits for a frame callback.
    pub fn wants_frame(&self) -> bool {
        let mut wants = false;
        for root in self.state.roots() {
            with_surface_tree_downward(
                &root,
                (),
                |_, _, &()| TraversalAction::DoChildren(()),
                |_, states, &()| {
                    wants |= !states
                        .cached_state
                        .get::<SurfaceAttributes>()
                        .current()
                        .frame_callbacks
                        .is_empty();
                },
                |_, _, &()| true,
            );
        }
        wants
    }

    /// Answers frame callbacks of every shown surface tree, after a frame was drawn.
    pub fn send_frame_callbacks(&mut self) {
        let time = self.state.start.elapsed().as_millis() as u32;
        for root in self.state.roots() {
            with_surface_tree_downward(
                &root,
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
    pub fn take_changed(&mut self) -> HashSet<Target> {
        std::mem::take(&mut self.changed)
    }

    fn root_of(&self, target: &Target) -> Option<WlSurface> {
        match target {
            Target::Screen(slot) => self.screens[*slot].as_ref().map(|t| t.wl_surface().clone()),
            Target::Popup(id) => self
                .popups
                .iter()
                .find(|p| &p.surface.wl_surface().id() == id)
                .map(|p| p.surface.wl_surface().clone()),
        }
    }

    /// Root surfaces of all windows and pop-ups.
    fn roots(&self) -> Vec<WlSurface> {
        self.screens
            .iter()
            .flatten()
            .map(|t| t.wl_surface().clone())
            .chain(self.popups.iter().map(|p| p.surface.wl_surface().clone()))
            .collect()
    }

    /// Screen, position in that window's pixels and depth of the pop-up or
    /// window with root `surface`.
    fn place_of(&self, surface: &WlSurface) -> Option<(usize, Point<i32, Logical>, u32)> {
        if let Some(slot) = self.screen_of(surface) {
            return Some((slot, (0, 0).into(), 0));
        }
        let popup = self
            .popups
            .iter()
            .find(|p| p.surface.wl_surface() == surface)?;
        let parent = popup.surface.get_parent_surface()?;
        let (slot, pos, _) = self.place_of(&parent)?;
        Some((slot, pos + popup.geometry.loc, popup.depth))
    }

    /// Pop-up geometry that stays within the window (plus margin) where possible.
    fn popup_geometry(
        &self,
        slot: usize,
        parent_pos: Point<i32, Logical>,
        positioner: PositionerState,
    ) -> Rectangle<i32, Logical> {
        let size = self
            .layout(&Target::Screen(slot))
            .map(|l| (l.size.0 as i32, l.size.1 as i32))
            .unwrap_or(WINDOW_SIZE);
        let target = Rectangle::new(
            (-POPUP_MARGIN - parent_pos.x, -POPUP_MARGIN - parent_pos.y).into(),
            (size.0 + 2 * POPUP_MARGIN, size.1 + 2 * POPUP_MARGIN).into(),
        );
        positioner.get_unconstrained_geometry(target)
    }
    /// Layout of `target`, or `None` if nothing is mapped.
    fn layout(&self, target: &Target) -> Option<WindowLayout> {
        let root = self.root_of(target)?;
        let root = &root;
        let mut surfaces = Vec::new();
        with_surface_tree_upward(
            root,
            (0, 0),
            |_, states, &(x, y)| {
                let view = states
                    .data_map
                    .get::<RendererSurfaceStateUserData>()
                    .and_then(|d| d.lock().unwrap().view());
                match view {
                    Some(view) => {
                        TraversalAction::DoChildren((x + view.offset.x, y + view.offset.y))
                    }
                    None => TraversalAction::SkipChildren,
                }
            },
            |surface, states, &(x, y)| {
                let Some(data) = states.data_map.get::<RendererSurfaceStateUserData>() else {
                    return;
                };
                let data = data.lock().unwrap();
                let (Some(view), Some(buffer_size)) = (data.view(), data.buffer_size()) else {
                    return;
                };
                let (bw, bh) = (buffer_size.w as f64, buffer_size.h as f64);
                let src = view.src;
                surfaces.push(SurfaceLayout {
                    surface: surface.clone(),
                    id: surface.id(),
                    commit: data.current_commit(),
                    dst: [
                        (x + view.offset.x) as f32,
                        (y + view.offset.y) as f32,
                        view.dst.w as f32,
                        view.dst.h as f32,
                    ],
                    uv: [
                        (src.loc.x / bw) as f32,
                        (src.loc.y / bh) as f32,
                        ((src.loc.x + src.size.w) / bw) as f32,
                        ((src.loc.y + src.size.h) / bh) as f32,
                    ],
                });
            },
            |_, _, _| true,
        );
        if surfaces.is_empty() {
            return None;
        }

        // Crop to the window geometry (drops client-side shadows), or the
        // bounding box of all surfaces if the client set none.
        let geometry = with_states(root, |s| {
            s.cached_state
                .get::<SurfaceCachedState>()
                .current()
                .geometry
        });
        let (x0, y0, w, h) = match geometry {
            Some(g) => (
                g.loc.x as f32,
                g.loc.y as f32,
                g.size.w as f32,
                g.size.h as f32,
            ),
            None => {
                let x0 = surfaces.iter().map(|s| s.dst[0]).fold(f32::MAX, f32::min);
                let y0 = surfaces.iter().map(|s| s.dst[1]).fold(f32::MAX, f32::min);
                let x1 = surfaces
                    .iter()
                    .map(|s| s.dst[0] + s.dst[2])
                    .fold(f32::MIN, f32::max);
                let y1 = surfaces
                    .iter()
                    .map(|s| s.dst[1] + s.dst[3])
                    .fold(f32::MIN, f32::max);
                (x0, y0, x1 - x0, y1 - y0)
            }
        };
        if w < 1.0 || h < 1.0 {
            return None;
        }
        for s in &mut surfaces {
            s.dst[0] -= x0;
            s.dst[1] -= y0;
        }
        Some(WindowLayout {
            size: (w as u32, h as u32),
            surfaces,
        })
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

    fn new_surface(&mut self, surface: &WlSurface) {
        add_pre_commit_hook::<Self, _>(surface, |state, _dh, surface| {
            state.block_until_ready(surface);
        });
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);

        let mut root = surface.clone();
        while let Some(parent) = get_parent(&root) {
            root = parent;
        }
        if let Some(popup) = self.popups.iter().find(|p| p.surface.wl_surface() == &root) {
            if !popup.surface.is_initial_configure_sent() {
                let _ = popup.surface.send_configure();
            }
            self.changed.insert(Target::Popup(root.id()));
            return;
        }
        let Some(slot) = self.screen_of(&root) else {
            return;
        };
        let toplevel = self.screens[slot].as_ref().unwrap();
        if !toplevel.is_initial_configure_sent() {
            toplevel.send_configure();
        }
        self.changed.insert(Target::Screen(slot));
    }
}

impl ShellState {
    /// Holds back a commit with a dmabuf until the client's GPU finished
    /// drawing it: via the explicit acquire point if the client set one,
    /// otherwise via the dmabuf's implicit fence.
    fn block_until_ready(&mut self, surface: &WlSurface) {
        let mut acquire_point = None;
        let dmabuf: Option<Dmabuf> = with_states(surface, |data| {
            acquire_point.clone_from(
                &data
                    .cached_state
                    .get::<DrmSyncobjCachedState>()
                    .pending()
                    .acquire_point,
            );
            data.cached_state
                .get::<SurfaceAttributes>()
                .pending()
                .buffer
                .as_ref()
                .and_then(|assignment| match assignment {
                    BufferAssignment::NewBuffer(buffer) => get_dmabuf(buffer).cloned().ok(),
                    _ => None,
                })
        });
        let Some(dmabuf) = dmabuf else {
            return;
        };
        let Some(client) = surface.client() else {
            return;
        };
        let explicit = acquire_point.is_some();
        if self.sync_logged.insert(surface.id()) {
            tracing::info!(
                "{}: Sync-Weg {}",
                surface.id(),
                if explicit {
                    "explizit (syncobj)"
                } else {
                    "implizit (dmabuf-Fence)"
                }
            );
        }
        match acquire_point {
            Some(point) => {
                if let Ok((blocker, source)) = point.generate_blocker() {
                    let inserted = self.loop_handle.insert_source(source, move |_, _, state| {
                        state.unblock(&client);
                        Ok(())
                    });
                    if inserted.is_ok() {
                        add_blocker(surface, blocker);
                    }
                }
            }
            None => {
                if let Ok((blocker, source)) = dmabuf.generate_blocker(Interest::READ) {
                    let inserted = self.loop_handle.insert_source(source, move |_, _, state| {
                        state.unblock(&client);
                        Ok(())
                    });
                    if inserted.is_ok() {
                        add_blocker(surface, blocker);
                    }
                }
            }
        }
    }

    fn unblock(&mut self, client: &Client) {
        let dh = self.display_handle.clone();
        self.client_compositor_state(client)
            .blocker_cleared(self, &dh);
    }
}

impl DmabufHandler for ShellState {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        let result = match &mut self.importer {
            Some(importer) => importer.import(&dmabuf),
            None => Err("kein Importer".into()),
        };
        match result {
            Ok(_) => {
                let _ = notifier.successful::<ShellState>();
            }
            Err(e) => {
                tracing::warn!("dmabuf abgelehnt: {e}");
                notifier.failed();
            }
        }
    }
}

impl DrmSyncobjHandler for ShellState {
    fn drm_syncobj_state(&mut self) -> Option<&mut DrmSyncobjState> {
        self.syncobj_state.as_mut()
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
            self.changed.insert(Target::Screen(slot));
        }
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        let Some((slot, parent_pos, parent_depth)) =
            surface.get_parent_surface().and_then(|p| self.place_of(&p))
        else {
            tracing::warn!("Pop-up ohne bekanntes Elternfenster");
            return;
        };
        let geometry = self.popup_geometry(slot, parent_pos, positioner);
        surface.with_pending_state(|state| state.geometry = geometry);
        tracing::info!(
            "Pop-up auf Screen {slot}, Ebene {}, {geometry:?}",
            parent_depth + 1
        );
        self.popups.push(PopupEntry {
            surface,
            slot,
            depth: parent_depth + 1,
            geometry,
            grab: false,
        });
    }

    fn grab(&mut self, surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        if let Some(popup) = self.popups.iter_mut().find(|p| p.surface == surface) {
            popup.grab = true;
        }
        // Menus are navigated by keyboard too.
        let keyboard = self.seat.get_keyboard().expect("seat has a keyboard");
        keyboard.set_focus(
            self,
            Some(surface.wl_surface().clone()),
            SERIAL_COUNTER.next_serial(),
        );
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        let Some((slot, parent_pos, _)) =
            surface.get_parent_surface().and_then(|p| self.place_of(&p))
        else {
            return;
        };
        let geometry = self.popup_geometry(slot, parent_pos, positioner);
        surface.with_pending_state(|state| state.geometry = geometry);
        if let Some(popup) = self.popups.iter_mut().find(|p| p.surface == surface) {
            popup.geometry = geometry;
        }
        surface.send_repositioned(token);
        let _ = surface.send_configure();
        self.changed
            .insert(Target::Popup(surface.wl_surface().id()));
    }

    fn popup_destroyed(&mut self, surface: PopupSurface) {
        self.popups.retain(|p| p.surface != surface);
        self.changed
            .insert(Target::Popup(surface.wl_surface().id()));
        // Give the keyboard back to the parent if the pop-up had it.
        let keyboard = self.seat.get_keyboard().expect("seat has a keyboard");
        if keyboard.current_focus().as_ref() == Some(surface.wl_surface())
            && let Some(parent) = surface.get_parent_surface()
        {
            keyboard.set_focus(self, Some(parent), SERIAL_COUNTER.next_serial());
        }
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

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor = Some(image);
    }
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
