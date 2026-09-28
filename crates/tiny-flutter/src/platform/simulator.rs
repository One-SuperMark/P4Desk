#[cfg(feature = "simulator")]
use crate::app::App;
#[cfg(feature = "simulator")]
use crate::graphics::geometry::{Point, Rect, Size};
#[cfg(feature = "simulator")]
use crate::platform::backend::PlatformBackend;
#[cfg(feature = "simulator")]
use crate::rendering::render_box::TouchEvent;
#[cfg(feature = "simulator")]
use crate::widgets::widget::Widget;
#[cfg(feature = "simulator")]
use std::num::NonZeroU32;
#[cfg(feature = "simulator")]
use std::sync::Arc;
#[cfg(feature = "simulator")]
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[cfg(feature = "simulator")]
pub struct SimulatorBackend {
    width: u32,
    height: u32,
    surface_buffer: Vec<u32>,
    pending_events: Vec<TouchEvent>,
}

#[cfg(feature = "simulator")]
impl SimulatorBackend {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            surface_buffer: vec![0xFF000000; (width * height) as usize],
            pending_events: Vec::new(),
        }
    }

    pub fn buffer(&self) -> &[u32] {
        &self.surface_buffer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.surface_buffer
            .resize((width * height) as usize, 0xFF000000);
        self.surface_buffer.fill(0xFF000000);
    }

    pub fn push_event(&mut self, event: TouchEvent) {
        self.pending_events.push(event);
    }
}

#[cfg(feature = "simulator")]
impl PlatformBackend for SimulatorBackend {
    fn screen_size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    fn flush(&mut self, rect: Rect, rgb565_data: &[u16]) {
        let x1 = (rect.x.floor() as i32).clamp(0, self.width as i32);
        let y1 = (rect.y.floor() as i32).clamp(0, self.height as i32);
        let x2 = (rect.right().ceil() as i32).clamp(x1, self.width as i32);
        let y2 = (rect.bottom().ceil() as i32).clamp(y1, self.height as i32);

        let rect_w = (x2 - x1) as usize;
        let rect_h = (y2 - y1) as usize;

        if rect_w == 0 || rect_h == 0 || rgb565_data.len() < rect_w * rect_h {
            return;
        }

        let mut in_idx = 0;
        for y in y1..y2 {
            let row_idx = (y as usize) * (self.width as usize);
            for x in x1..x2 {
                let px565 = rgb565_data[in_idx];
                in_idx += 1;

                // Expand RGB565 back to 0x00RRGGBB for presentation on desktop
                let r = (((px565 >> 11) & 0x1F) * 255 / 31) as u32;
                let g = (((px565 >> 5) & 0x3F) * 255 / 63) as u32;
                let b = ((px565 & 0x1F) * 255 / 31) as u32;

                let out_idx = row_idx + (x as usize);
                if out_idx < self.surface_buffer.len() {
                    self.surface_buffer[out_idx] = (r << 16) | (g << 8) | b;
                }
            }
        }
    }

    fn poll_touch(&mut self) -> Option<TouchEvent> {
        if self.pending_events.is_empty() {
            None
        } else {
            Some(self.pending_events.remove(0))
        }
    }

    fn set_screen_power(&mut self, on: bool) {
        log::info!("[Simulator] Physical screen power set to: {}", on);
        if !on {
            self.surface_buffer.fill(0xFF000000);
        }
    }
}

/// Helper to run an interactive desktop window simulation of a tiny-flutter widget with a reactive builder.
/// Supports dynamic window resizing with responsive layout adaptation.
#[cfg(feature = "simulator")]
pub fn run_simulator<F, W, M>(
    title: &str,
    width: u32,
    height: u32,
    builder: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: crate::app::ResponsiveBuilder<M, Output = W> + 'static,
    W: Widget + 'static,
{
    run_simulator_internal(title, width, height, builder, false)
}

/// Helper to run an interactive desktop window simulation with continuous ~60fps animation updates.
#[cfg(feature = "simulator")]
pub fn run_simulator_animated<F, W, M>(
    title: &str,
    width: u32,
    height: u32,
    builder: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: crate::app::ResponsiveBuilder<M, Output = W> + 'static,
    W: Widget + 'static,
{
    run_simulator_internal(title, width, height, builder, true)
}

#[cfg(feature = "simulator")]
// Keep the original closure-oriented runner through winit 0.30's compatibility entry points.
#[allow(deprecated)]
fn run_simulator_internal<F, W, M>(
    title: &str,
    width: u32,
    height: u32,
    mut builder: F,
    animated: bool,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: crate::app::ResponsiveBuilder<M, Output = W> + 'static,
    W: Widget + 'static,
{
    let event_loop = EventLoop::new()?;
    let title_string = format!("{} ({}x{})", title, width, height);
    let window = Arc::new(
        event_loop.create_window(
            Window::default_attributes()
                .with_title(&title_string)
                .with_inner_size(LogicalSize::new(width as f64, height as f64))
                .with_min_inner_size(LogicalSize::new(180.0, 180.0))
                .with_resizable(true),
        )?,
    );

    let context = softbuffer::Context::new(window.clone())
        .map_err(|e| format!("Failed to create softbuffer context: {e}"))?;
    let mut surface = softbuffer::Surface::new(&context, window.clone())
        .map_err(|e| format!("Failed to create softbuffer surface: {e}"))?;

    let mut phys_size = window.inner_size();
    surface
        .resize(
            NonZeroU32::new(phys_size.width.max(1)).unwrap(),
            NonZeroU32::new(phys_size.height.max(1)).unwrap(),
        )
        .map_err(|e| format!("Failed to resize surface: {e}"))?;

    let scale_factor = window.scale_factor();
    let logical = phys_size.to_logical::<f64>(scale_factor);
    let mut cur_w = (logical.width.round() as u32).max(1);
    let mut cur_h = (logical.height.round() as u32).max(1);

    let mut backend = SimulatorBackend::new(cur_w, cur_h);
    let initial_widget = builder.build(Size::new(cur_w as f32, cur_h as f32));
    let mut app = App::new(initial_widget, Size::new(cur_w as f32, cur_h as f32));

    let mut cursor_pos = Point::ZERO;
    let mut is_mouse_down = false;

    let mut next_frame = std::time::Instant::now();
    window.request_redraw();

    match event_loop.run(move |event, target| {
        if animated {
            target.set_control_flow(ControlFlow::WaitUntil(next_frame));
        } else {
            target.set_control_flow(ControlFlow::Wait);
        }

        match event {
            Event::AboutToWait if animated => {
                let now = std::time::Instant::now();
                if now >= next_frame {
                    window.request_redraw();
                    next_frame = now + std::time::Duration::from_millis(16);
                }
            }
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => {
                    target.exit();
                }
                WindowEvent::Resized(new_size) => {
                    phys_size = new_size;
                    if let (Some(w), Some(h)) = (
                        NonZeroU32::new(new_size.width.max(1)),
                        NonZeroU32::new(new_size.height.max(1)),
                    ) {
                        let _ = surface.resize(w, h);
                    }

                    let scale_factor = window.scale_factor();
                    let logical_size = new_size.to_logical::<f64>(scale_factor);
                    let new_w = (logical_size.width.round() as u32).max(1);
                    let new_h = (logical_size.height.round() as u32).max(1);

                    if new_w != cur_w || new_h != cur_h {
                        cur_w = new_w;
                        cur_h = new_h;
                        window.set_title(&format!("{} ({}x{})", title, cur_w, cur_h));
                        backend.resize(cur_w, cur_h);
                        let screen_size = Size::new(cur_w as f32, cur_h as f32);
                        app.resize(screen_size);
                        app.set_root(builder.build(screen_size));
                    }
                    window.request_redraw();
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    let new_size = window.inner_size();
                    phys_size = new_size;
                    if let (Some(w), Some(h)) = (
                        NonZeroU32::new(new_size.width.max(1)),
                        NonZeroU32::new(new_size.height.max(1)),
                    ) {
                        let _ = surface.resize(w, h);
                    }

                    let logical_size = new_size.to_logical::<f64>(scale_factor);
                    let new_w = (logical_size.width.round() as u32).max(1);
                    let new_h = (logical_size.height.round() as u32).max(1);

                    if new_w != cur_w || new_h != cur_h {
                        cur_w = new_w;
                        cur_h = new_h;
                        window.set_title(&format!("{} ({}x{})", title, cur_w, cur_h));
                        backend.resize(cur_w, cur_h);
                        let screen_size = Size::new(cur_w as f32, cur_h as f32);
                        app.resize(screen_size);
                        app.set_root(builder.build(screen_size));
                    }
                    window.request_redraw();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let scale_factor = window.scale_factor();
                    let logical_pos = position.to_logical::<f64>(scale_factor);
                    let virt_x = (logical_pos.x as f32).clamp(0.0, (cur_w.saturating_sub(1)) as f32);
                    let virt_y = (logical_pos.y as f32).clamp(0.0, (cur_h.saturating_sub(1)) as f32);

                    cursor_pos = Point::new(virt_x, virt_y);
                    if is_mouse_down {
                        backend.push_event(TouchEvent::Move(cursor_pos));
                        window.request_redraw();
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left {
                        match state {
                            ElementState::Pressed => {
                                is_mouse_down = true;
                                log::debug!("[Simulator] Click Down at ({:.1}, {:.1})", cursor_pos.x, cursor_pos.y);
                                backend.push_event(TouchEvent::Down(cursor_pos));
                                window.request_redraw();
                            }
                            ElementState::Released => {
                                is_mouse_down = false;
                                log::debug!("[Simulator] Click Up at ({:.1}, {:.1})", cursor_pos.x, cursor_pos.y);
                                backend.push_event(TouchEvent::Up(cursor_pos));
                                window.request_redraw();
                            }
                        }
                    }
                }
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    if key_event.state == ElementState::Pressed {
                        match &key_event.logical_key {
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => {
                                log::info!("[Simulator] Button 1 pressed (Escape) -> HardwareBack (Background)");
                                backend.push_event(TouchEvent::HardwareBack);
                                window.request_redraw();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F2)
                            | winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
                                log::info!("[Simulator] Button 2 pressed (Power Key: F2/Space) -> HardwarePower (Screen Off/On)");
                                backend.push_event(TouchEvent::HardwarePower);
                                window.request_redraw();
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete)
                            | winit::keyboard::Key::Named(winit::keyboard::NamedKey::F3) => {
                                log::info!("[Simulator] Button 3 pressed (Delete/F3) -> HardwareKill (Kill & Clear State)");
                                backend.push_event(TouchEvent::HardwareKill);
                                window.request_redraw();
                            }
                            winit::keyboard::Key::Character(s) => {
                                if s.eq_ignore_ascii_case("q") {
                                    log::info!("[Simulator] Button 1 pressed ('Q') -> HardwareBack (Background)");
                                    backend.push_event(TouchEvent::HardwareBack);
                                    window.request_redraw();
                                } else if s.eq_ignore_ascii_case("e") {
                                    log::info!("[Simulator] Button 3 pressed ('E') -> HardwareKill (Kill & Clear State)");
                                    backend.push_event(TouchEvent::HardwareKill);
                                    window.request_redraw();
                                } else if s.eq_ignore_ascii_case("b") {
                                    log::info!("[Simulator] Button 1 pressed ('B') -> HardwareBack (Background)");
                                    backend.push_event(TouchEvent::HardwareBack);
                                    window.request_redraw();
                                } else if s.eq_ignore_ascii_case("p") {
                                    log::info!("[Simulator] Button 2 pressed ('P') -> HardwarePower (Screen Off/On)");
                                    backend.push_event(TouchEvent::HardwarePower);
                                    window.request_redraw();
                                }
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    if animated {
                        app.step_animated(&mut backend, |sz| builder.build(sz));
                    } else {
                        app.step_with_builder(&mut backend, |sz| builder.build(sz));
                    }

                    if let Ok(mut buffer) = surface.buffer_mut() {
                        let pw = phys_size.width as usize;
                        let ph = phys_size.height as usize;
                        let vw = cur_w as usize;
                        let vh = cur_h as usize;

                        let src_pixels = backend.buffer();

                        if pw == vw && ph == vh {
                            let len = pw * ph;
                            if src_pixels.len() >= len && buffer.len() >= len {
                                buffer[..len].copy_from_slice(&src_pixels[..len]);
                            }
                        } else if pw == vw * 2 && ph == vh * 2 {
                            // Ultra-fast 2x integer upscale: copy each pixel into 2x2 blocks
                            // Razor-sharp characters and symbols, zero blur, sub-millisecond execution!
                            for y in 0..vh {
                                let src_row = y * vw;
                                let dst_row0 = (y * 2) * pw;
                                let dst_row1 = dst_row0 + pw;
                                for x in 0..vw {
                                    let px = src_pixels[src_row + x];
                                    let dst_x = x * 2;
                                    buffer[dst_row0 + dst_x] = px;
                                    buffer[dst_row0 + dst_x + 1] = px;
                                    buffer[dst_row1 + dst_x] = px;
                                    buffer[dst_row1 + dst_x + 1] = px;
                                }
                            }
                        } else {
                            let scale_x = pw as f32 / vw.max(1) as f32;
                            let scale_y = ph as f32 / vh.max(1) as f32;

                            for py in 0..ph {
                                let sy = ((py as f32 / scale_y) as usize).min(vh.saturating_sub(1));
                                let src_row = sy * vw;
                                let dst_row = py * pw;
                                for px in 0..pw {
                                    let sx = ((px as f32 / scale_x) as usize).min(vw.saturating_sub(1));
                                    if dst_row + px < buffer.len() && src_row + sx < src_pixels.len() {
                                        buffer[dst_row + px] = src_pixels[src_row + sx];
                                    }
                                }
                            }
                        }

                        let _ = buffer.present();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }) {
        Ok(()) => Ok(()),
        Err(winit::error::EventLoopError::ExitFailure(_)) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
