use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

mod schmising;

const W: usize = 1024;
const H: usize = 512;

#[macroquad::main("schmising")]
async fn main() {
    let mut mat = schmising::MagnetizedMaterial::<W, H>::new_halfhalf(0.2);

    let mut image = Image::gen_image_color(W.try_into().unwrap(), H.try_into().unwrap(), BLACK);

    let texture = Texture2D::from_image(&image);

    // Zoom and pan state
    let mut zoom = 1.0f32;
    let mut camera_x = 0.0f32;
    let mut camera_y = 0.0f32;
    let mut last_mouse_pos: Option<(f32, f32)> = None;
    
    // Log scale sliders (store log values)
    let mut log_temperature = mat.temperature.ln();
    let mut log_steps_per_frame = (10000.0f32).ln();

    loop {
        clear_background(BLACK);

        // Handle panning with left mouse button drag (but not when over UI)
        if is_mouse_button_down(MouseButton::Left) && !root_ui().is_mouse_over(mouse_position().into()) {
            let current_mouse = mouse_position();
            if let Some(last_pos) = last_mouse_pos {
                camera_x += current_mouse.0 - last_pos.0;
                camera_y += current_mouse.1 - last_pos.1;
            }
            last_mouse_pos = Some(current_mouse);
        } else {
            last_mouse_pos = None;
        }

        // Handle zoom with mouse wheel
        let mouse_wheel = mouse_wheel().1;
        if mouse_wheel != 0.0 {
            let mouse_pos = mouse_position();
            let old_zoom = zoom;
            
            // Zoom in/out
            zoom *= 1.0 + mouse_wheel * 0.1;
            zoom = zoom.clamp(0.1, 10.0);
            
            // Adjust camera to zoom towards mouse position
            let zoom_factor = zoom / old_zoom;
            camera_x = mouse_pos.0 + (camera_x - mouse_pos.0) * zoom_factor;
            camera_y = mouse_pos.1 + (camera_y - mouse_pos.1) * zoom_factor;
        }

        let max_steps_per_frame = log_steps_per_frame.exp() as usize;
        let sim_start = get_time();
        let mut actual_steps = 0;
        // Allow up to 10ms for simulation to maintain ~60fps
        while actual_steps < max_steps_per_frame && (get_time() - sim_start) < 0.01 {
            mat.step();
            actual_steps += 1;
        }

        for r in 0..H {
            for c in 0..W {
                let spin = mat.get_cell(r as isize, c as isize);
                let color = match spin {
                    schmising::Spin::Down => BLUE,
                    schmising::Spin::Up => RED,
                    schmising::Spin::OOB => GREEN
                };
                image.set_pixel(c as u32, r as u32, color);
            }
        }

        texture.update(&image);
        
        // Draw texture with zoom and pan
        let dest_width = W as f32 * zoom;
        let dest_height = H as f32 * zoom;
        draw_texture_ex(
            &texture,
            camera_x,
            camera_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(dest_width, dest_height)),
                ..Default::default()
            },
        );
        
        // Temperature slider UI
        widgets::Window::new(hash!(), vec2(10., 10.), vec2(300., 120.))
            .label("Controls")
            .ui(&mut root_ui(), |ui| {
                ui.label(None, &format!("Temperature: {:.4}", mat.temperature));
                ui.slider(hash!(), "Temp (log)", -5.0f32..3.0f32, &mut log_temperature);
                mat.temperature = log_temperature.exp();
                
                ui.label(None, &format!("Steps/frame: {} (max {})", actual_steps, max_steps_per_frame));
                ui.slider(hash!(), "Speed (log)", 2.0f32..14.0f32, &mut log_steps_per_frame);
            });
        
        next_frame().await
    }
}
