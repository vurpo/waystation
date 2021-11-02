use std::{borrow::{Borrow, Cow}, ops::Deref, process::Command, sync::mpsc::Receiver};

use conrod_core::{Colorable, Positionable, Widget, widget, widget_ids};
use gilrs::{Button, EventType, Gilrs};
use glium::Surface;
use smithay::reexports::calloop::channel::Sender;

conrod_winit::v023_conversion_fns!();

mod drawing;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;

struct UiState {
    menu_on_top: bool
}

pub enum ToCompositor {
    SetMenuOnTop(bool)
}

pub enum ToUi {
    SetMenuOnTop(bool)
}

pub fn main(rx: Receiver<ToUi>, tx: Sender<ToCompositor>) {
    // Build the window.
    let event_loop: glium::glutin::event_loop::EventLoop<()> = glium::glutin::platform::unix::EventLoopExtUnix::new_wayland_any_thread();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Hello Conrod!")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    let mut ui_state = UiState {
        menu_on_top: true
    };

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate the widget identifiers.
    widget_ids!(struct Ids { text, list });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    let mut gilrs = Gilrs::new().unwrap();
    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let applications = vec![
        ("RetroArch", "retroarch"),
        ("SuperTuxKart", "supertuxkart"),
        ("dummy application", ""),
        ("dummy application", ""),
    ];

    let mut selected_application: usize = 0;

    let mut should_update_ui = true;
    event_loop.run(move |event, _, control_flow| {
        // Break from the loop upon `Escape` or closed window.
        match &event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the loop upon `Escape`.
                glium::glutin::event::WindowEvent::CloseRequested
                | glium::glutin::event::WindowEvent::KeyboardInput {
                    input:
                        glium::glutin::event::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }

        while let Ok(event) = rx.try_recv() {
            match event {
                ToUi::SetMenuOnTop(on_top) => ui_state.menu_on_top = on_top,
            }
        }

        // Check gamepad input
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            //active_gamepad = Some(id);
            if let EventType::ButtonPressed(Button::Mode, _) = event {
                should_update_ui = true;
                ui_state.menu_on_top = !ui_state.menu_on_top;
                tx.send(ToCompositor::SetMenuOnTop(ui_state.menu_on_top));
            } else if ui_state.menu_on_top {
                should_update_ui = true;
                match event {
                    EventType::ButtonPressed(Button::DPadUp, _) | EventType::ButtonRepeated(Button::DPadUp, _) => {
                        if selected_application == 0 { selected_application = applications.len()-1; }
                        else { selected_application -= 1; }
                    }
                    EventType::ButtonPressed(Button::DPadDown, _) | EventType::ButtonRepeated(Button::DPadDown, _) => {
                        if selected_application == applications.len()-1 { selected_application = 0; }
                        else { selected_application += 1; }
                    }
                    EventType::ButtonPressed(Button::East, _) => {
                        Command::new(applications[selected_application].1).spawn();
                    }
                    _ => {
                        should_update_ui = false;
                    }
                } 
            }
        }

        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = convert_event(&event, &display.gl_window().window()) {
            ui.handle_event(event);
            should_update_ui = true;
        }

        match &event {
            glium::glutin::event::Event::MainEventsCleared => {
                if should_update_ui {
                    should_update_ui = false;

                    // Set the widgets.
                    let ui = &mut ui.set_widgets();

                    let (mut items, scrollbar) = widget::List::flow_down(applications.len())
                        .top_left_of(ui.window)
                        .set(ids.list, ui);
                    
                    while let Some(item) = items.next(ui) {
                        let i = item.i;

                        let text = format!("{}{}", if i == selected_application {"> "} else {""}, applications[i].0);

                        let label = widget::Text::new(&text);
                        item.set(label, ui);
                    }

                    // "Hello World!" in the middle of the screen.
/*                      widget::Text::new("Hello World!")
                        .middle_of(ui.window)
                        .color(conrod_core::color::BLACK)
                        .font_size(32)
                        .set(ids.text, ui); */

                    // Request redraw if the `Ui` has changed.
                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                // Draw the `Ui` if it has changed.
                let primitives = ui.draw();

                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(1.0, 1.0, 1.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
            _ => {}
        }
    })
}