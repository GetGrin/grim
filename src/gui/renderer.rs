// Copyright 2023 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::iter;
use std::time::Instant;
use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use log::{debug, error, trace, warn};
use wgpu::{CompositeAlphaMode, InstanceDescriptor};
use winit::event::Event::*;
use winit::event::StartCause;
use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::platform::run_return::EventLoopExtRunReturn;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    RequestRedraw,
}

pub fn start(mut event_loop: EventLoop<Event>) {
    let window = winit::window::WindowBuilder::new()
        .with_transparent(false)
        .with_title("Grim")
        .build(&event_loop)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to init window at {} line {} with error\n{:?}",
                file!(),
                line!(),
                e
            )
        });

    let instance_descriptor = InstanceDescriptor::default();

    let mut instance = wgpu::Instance::new(instance_descriptor);

    let mut size = window.inner_size();
    let outer_size = window.outer_size();

    warn!("outer_size = {:?}", outer_size);
    warn!("size = {:?}", size);

    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    #[cfg(target_os = "android")]
    let mut platform = {
        event_loop.run_return(|main_event, tgt, control_flow| {
            control_flow.set_poll();
            warn!(
                "Got event: {:?} at {} line {}",
                &main_event,
                file!(),
                line!()
            );
            match main_event {
                NewEvents(e) => match e {
                    StartCause::ResumeTimeReached { .. } => {}
                    StartCause::WaitCancelled { .. } => {}
                    StartCause::Poll => {}
                    StartCause::Init => {}
                },
                WindowEvent {
                    window_id,
                    ref event,
                } => {
                    if let winit::event::WindowEvent::Resized(r) = event {
                        size = *r;
                    }
                }
                DeviceEvent { .. } => {}
                UserEvent(_) => {}
                Suspended => {
                    control_flow.set_poll();
                }
                Resumed => {
                    if let Some(primary_mon) = tgt.primary_monitor() {
                        size = primary_mon.size();
                        window.set_inner_size(size);
                        warn!(
                            "Set to new size: {:?} at {} line {}",
                            &size,
                            file!(),
                            line!()
                        );
                    } else if let Some(other_mon) = tgt.available_monitors().next() {
                        size = other_mon.size();
                        window.set_inner_size(size);
                        warn!(
                            "Set to new size: {:?} at {} line {}",
                            &size,
                            file!(),
                            line!()
                        );
                    }
                    control_flow.set_exit();
                }
                MainEventsCleared => {}
                RedrawRequested(rdr) => {}
                RedrawEventsCleared => {}
                LoopDestroyed => {}
            };
            platform.handle_event(&main_event);
        });

        Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        })
    };


    warn!("WGPU new surface at {} line {}", file!(), line!());
    let mut surface = unsafe { instance.create_surface(&window).unwrap_or_else(|e| {
        panic!(
            "Failed to create surface at {} line {} with error\n{:?}",
            file!(),
            line!(),
            e
        )
    }) };

    warn!("instance request_adapter at {} line {}", file!(), line!());
    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
        .unwrap_or_else(|| panic!("Failed get adapter at {} line {}", file!(), line!()));

    warn!("adapter request_device at {} line {}", file!(), line!());
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::downlevel_defaults(),
            label: None,
        },
        None,
    ))
        .unwrap_or_else(|e| {
            panic!(
                "Failed to request device at {} line {} with error\n{:?}",
                file!(),
                line!(),
                e
            )
        });

    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities.formats[0];
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![surface_format],
    };

    warn!("surface configure at {} line {}", file!(), line!());
    surface.configure(&device, &surface_config);

    warn!("RenderPass new at {} line {}", file!(), line!());
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);

    warn!("DemoWindows default at {} line {}", file!(), line!());
    // Display the demo application that ships with egui.
    let mut demo_app = egui_demo_lib::DemoWindows::default();

    let start_time = Instant::now();

    let mut in_bad_state = false;

    warn!("Enter the loop");
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        debug!("Got event: {:?} at {} line {}", &event, file!(), line!());
        platform.handle_event(&event);
        match event {
            RedrawRequested(..) => {
                platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        error!("The underlying surface has changed, and therefore the swap chain must be updated");
                        in_bad_state = true;
                        return;
                    }
                    Err(wgpu::SurfaceError::Lost) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        error!("LOST surface, drop frame. Originally: \"The swap chain has been lost and needs to be recreated\"");
                        in_bad_state = true;
                        return;
                    }
                    Err(e) => {
                        error!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Begin to draw the UI frame.
                platform.begin_frame();

                // Draw the demo application.
                demo_app.ui(&platform.context());

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let full_output = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(full_output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // Upload all resources for the GPU.
                let screen_descriptor = ScreenDescriptor  {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32
                };
                let tdelta: egui::TexturesDelta = full_output.textures_delta;
                egui_rpass
                    .add_textures(&device, &queue, &tdelta)
                    .expect("add texture ok");
                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::BLACK),
                    )
                    .unwrap_or_else(|e| panic!("Failed to render pass at {} line {} with error\n{:?}", file!(), line!(), e));
                // Submit the commands.
                queue.submit(iter::once(encoder.finish()));

                // Redraw egui
                output_frame.present();

                egui_rpass
                    .remove_textures(tdelta)
                    .expect("remove texture ok");

                // Support reactive on windows only, but not on linux.
                // if _output.needs_repaint {
                //     *control_flow = ControlFlow::Poll;
                // } else {
                //     *control_flow = ControlFlow::Wait;
                // }
            }
            MainEventsCleared | UserEvent(Event::RequestRedraw) => {
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                winit::event::WindowEvent::Focused(focused) => {
                    in_bad_state |= !focused;
                },
                _ => {}
            },
            Resumed => {
                if in_bad_state {
                    //https://github.com/gfx-rs/wgpu/issues/2302
                    warn!("WGPU new surface at {} line {}", file!(), line!());
                    surface = unsafe { instance.create_surface(&window).unwrap_or_else(|e| {
                        panic!(
                            "Failed to create surface at {} line {} with error\n{:?}",
                            file!(),
                            line!(),
                            e
                        )
                    }) };
                    warn!("surface configure at {} line {}", file!(), line!());
                    surface.configure(&device, &surface_config);
                    in_bad_state = false;
                }
            },
            Suspended => (),
            _ => (),
        }
    });
}