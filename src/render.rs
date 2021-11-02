use slog::Logger;
use smithay::{
    backend::{
        renderer::{
            gles2::{Gles2Frame, Gles2Renderer},
            Frame,
        },
        SwapBuffersError,
    },
    utils::{Logical, Rectangle},
    wayland::shell::wlr_layer::Layer,
};

use crate::{drawing::{draw_layers, draw_top_window, draw_windows}, window_map::WindowMap};

pub fn render_layers_and_windows(
    renderer: &mut Gles2Renderer,
    frame: &mut Gles2Frame,
    window_map: &WindowMap,
    output_geometry: Rectangle<i32, Logical>,
    output_scale: f32,
    logger: &Logger,
) -> Result<(), SwapBuffersError> {

    frame.clear([0.0, 0.0, 0.0, 1.0])?;

    for layer in [Layer::Background, Layer::Bottom] {
        draw_layers(
            renderer,
            frame,
            window_map,
            layer,
            output_geometry,
            output_scale,
            logger,
        )?;
    }

    draw_top_window(renderer, frame, window_map, output_geometry, output_scale, logger)?;

    for layer in [Layer::Top, Layer::Overlay] {
        draw_layers(
            renderer,
            frame,
            window_map,
            layer,
            output_geometry,
            output_scale,
            logger,
        )?;
    }

    Ok(())
}
