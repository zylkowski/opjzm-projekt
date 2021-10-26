use std::{io::repeat, ops::Range};

use egui::{Color32, Painter, TextureId, Vec2};
use image::GrayAlphaImage;
use plotters::{prelude::*, style::RGBAColor};

#[macro_use]
extern crate glium;
use glium::glutin;
// use egui::*;

const WINDOW_SIZE: (u32, u32) = (1530, 520);
const CHART_SIZE: (usize, usize) = (650, 500);

struct ChartConfig {
    a: usize,
    z_big: usize,

    z: f64,
    // beta_gamma: f64,
    t_max: f64,
    delta: f64,

    bethe_min_x: f64,
    bethe_max_x: f64,

    bethe_min_y: f64,
    bethe_max_y: f64,

    energy_min_x: f64,
    energy_max_x: f64,

    energy_min_y: f64,
    energy_max_y: f64,
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: WINDOW_SIZE.0,
            height: WINDOW_SIZE.1,
        })
        .with_title("Bethe Bloch");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, event_loop).unwrap()
}

fn draw_bethe(chart_info: &ChartConfig, painter: &mut egui_glium::Painter, texture_id: TextureId) {
    let mut png_data: Vec<u8> = vec![0; CHART_SIZE.0 * CHART_SIZE.1 * 3];
    {
        let root_area =
            BitMapBackend::with_buffer(&mut png_data, (CHART_SIZE.0 as u32, CHART_SIZE.1 as u32))
                .into_drawing_area();
        root_area.fill(&RGBColor(40, 40, 40)).unwrap();
        let log_range = LogCoord::from(LogRange(chart_info.bethe_min_x..chart_info.bethe_max_x));

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin_left(20)
            .margin_right(20)
            .margin_bottom(10)
            .caption("Bethe-Bloch", ("sans-serif", 40, &WHITE))
            .build_cartesian_2d(log_range, chart_info.bethe_min_y..chart_info.bethe_max_y)
            .unwrap();

        //

        let resolution = 200.;
        let points = (chart_info.bethe_max_x - chart_info.bethe_min_x) * resolution;
        let range = (0..=(points as i128));

        ctx.draw_series(LineSeries::new(
            (range).map(|x| {
                let real_x = chart_info.bethe_min_x + (x as f64) / resolution;
                let value = stopping_power(
                    chart_info.a,
                    chart_info.z_big,
                    chart_info.z,
                    real_x,
                    chart_info.t_max,
                    chart_info.delta,
                );
                (real_x, value)
            }),
            &RED,
        ))
        .unwrap();

        ctx.configure_mesh()
            .bold_line_style(&WHITE)
            .axis_style(&WHITE)
            .y_desc("dE/dx [MeV*cm2/g")
            .x_desc("βγ")
            .label_style(("sans-serif", 15, &WHITE))
            .draw()
            .unwrap();
    }

    let tex: Vec<_> = png_data
        .chunks(3)
        .map(|pixel| Color32::from_rgb(pixel[0], pixel[1], pixel[2]))
        .collect();
    let image_size = Vec2::new(CHART_SIZE.0 as f32, CHART_SIZE.1 as f32);
    painter.set_user_texture(texture_id, (CHART_SIZE.0, CHART_SIZE.1), &tex[..]);
}

fn draw_energy(chart_info: &ChartConfig, painter: &mut egui_glium::Painter, texture_id: TextureId) {
    let mut png_data: Vec<u8> = vec![0; CHART_SIZE.0 * CHART_SIZE.1 * 3];
    {
        let root_area =
            BitMapBackend::with_buffer(&mut png_data, (CHART_SIZE.0 as u32, CHART_SIZE.1 as u32))
                .into_drawing_area();
        root_area.fill(&RGBColor(40, 40, 40)).unwrap();

        let mut ctx = ChartBuilder::on(&root_area)
            .set_label_area_size(LabelAreaPosition::Left, 80)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .margin_left(20)
            .margin_right(20)
            .margin_bottom(10)
            .caption("Energy and βγ relation", ("sans-serif", 40, &WHITE))
            .build_cartesian_2d(
                chart_info.energy_min_x..chart_info.energy_max_x,
                chart_info.energy_min_y..chart_info.energy_max_y,
            )
            .unwrap();

        let resolution = 200.;
        let points = (chart_info.energy_max_x - chart_info.energy_min_x) * resolution;
        let range = (0..=(points as i128));

        // Electron, Muon, Pi, D, alpha
        let particles = vec![
            (0.511, &CYAN, "Electron"),
            (207. * 0.511, &MAGENTA, "Muon"),
            (273. * 0.511, &YELLOW, "Pi"),
            (1836. * 0.511, &BLUE, "Proton"),
            (3649. * 0.511, &GREEN, "D"),
            (7294. * 0.511, &RED, "Alpha"),
        ];

        for particle in particles {
            ctx.draw_series(LineSeries::new(
                (range.clone()).map(|x| {
                    let real_x = chart_info.energy_min_x + (x as f64) / resolution;
                    let value = e_kin(real_x, particle.0);
                    // println!(
                    //     "real_x: {} ekin: {}, particle_mas {}",
                    //     real_x, value, particle.0
                    // );
                    (real_x, value)
                }),
                particle.1,
            ))
            .unwrap()
            .label(particle.2)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], particle.1));
        }

        ctx.configure_mesh()
            .bold_line_style(&WHITE)
            .axis_style(&WHITE)
            .y_desc("E [MeV]")
            .x_desc("βγ")
            .label_style(("sans-serif", 15, &WHITE))
            .draw()
            .unwrap();

        ctx.configure_series_labels()
            .border_style(&WHITE)
            .background_style(&BLACK)
            .label_font(("sans-serif", 15, &WHITE))
            .draw()
            .unwrap();
    }

    let tex: Vec<_> = png_data
        .chunks(3)
        .map(|pixel| Color32::from_rgb(pixel[0], pixel[1], pixel[2]))
        .collect();
    let image_size = Vec2::new(CHART_SIZE.0 as f32, CHART_SIZE.1 as f32);
    painter.set_user_texture(texture_id, (CHART_SIZE.0, CHART_SIZE.1), &tex[..]);
}

fn e_kin(beta_gamma: f64, mass: f64) -> f64 {
    let beta = (beta_gamma.powi(2) / (beta_gamma.powi(2) + 1.)).sqrt();
    let gamma = beta_gamma / beta;

    (mass * (gamma - 1.))
}

fn stopping_power(a: usize, z_big: usize, z: f64, beta_gamma: f64, t_max: f64, delta: f64) -> f64 {
    const K: f64 = 0.3072;
    const M_e: f64 = 0.511;
    // const C: f64 = 1.;
    const C: f64 = 300000000.;

    let i = 10. * z_big as f64;
    let beta_2 = beta_gamma.powi(2) / (beta_gamma.powi(2) + 1.);

    K * ((z_big as f64) / (a as f64 * beta_2)) * ((w_m(beta_2) / i).ln() - beta_2)
}

fn w_m(beta_2: f64) -> f64 {
    const C: f64 = 300000000.;
    const C: f64 = 1.;
    const M_e: f64 = 0.511;
    println!("W_m = {}", (2. * M_e * C.powi(2) * beta_2) / (1. - beta_2));
    (2. * M_e * C.powi(2) * beta_2) / (1. - beta_2)
}

fn main() {
    // print!("E_kin prot 1 betagamma = {}", e_kin(1., 1836. * 0.511));

    // return;
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&event_loop);

    let mut egui = egui_glium::EguiGlium::new(&display);

    let mut chart_info = ChartConfig {
        a: 1,
        z_big: 1,
        z: 1.,
        t_max: 10.,
        delta: 10.,

        bethe_max_x: 1000.,
        bethe_min_x: 0.,

        bethe_min_y: 0.,
        bethe_max_y: 200.,

        energy_min_x: 0.,
        energy_max_x: 5.,

        energy_min_y: 0.,
        energy_max_y: 800.,
    };

    let (_, painter) = egui.ctx_and_painter_mut();
    let bethe_texture_id = painter.alloc_user_texture();
    draw_bethe(&chart_info, painter, bethe_texture_id);
    let energy_texture_id = painter.alloc_user_texture();
    draw_energy(&chart_info, painter, energy_texture_id);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            egui.begin_frame(&display);

            let mut quit = false;

            let mut recalculate_chart: bool = false;
            egui::SidePanel::left("left_panel")
                .resizable(false)
                .show(egui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Label::new("Bethe Bloch controls").strong());
                    });
                    ui.horizontal(|ui| {
                        ui.label("A: ");
                        recalculate_chart = ui
                            .add(egui::DragValue::new(&mut chart_info.a).clamp_range(1..=300))
                            .changed()
                            || recalculate_chart;
                        ui.label("Z: ");
                        recalculate_chart = ui
                            .add(egui::DragValue::new(&mut chart_info.z_big).clamp_range(1..=300))
                            .changed()
                            || recalculate_chart;
                    });

                    ui.separator();
                    ui.label("X axis");
                    ui.horizontal(|ui| {
                        ui.label("min: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.bethe_min_x)
                                    .clamp_range(0. ..=chart_info.bethe_max_x - 1.),
                            )
                            .changed()
                            || recalculate_chart;
                        ui.label("max: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.bethe_max_x)
                                    .clamp_range(1. ..=f64::INFINITY),
                            )
                            .changed()
                            || recalculate_chart;
                    });
                    ui.label("y axis");
                    ui.horizontal(|ui| {
                        ui.label("min: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.bethe_min_y)
                                    .clamp_range(0. ..=chart_info.bethe_max_x - 1.),
                            )
                            .changed()
                            || recalculate_chart;
                        ui.label("max: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.bethe_max_y)
                                    .clamp_range((chart_info.bethe_min_y + 0.1)..=f64::INFINITY),
                            )
                            .changed()
                            || recalculate_chart;
                    });

                    ui.separator();

                    ui.vertical_centered(|ui| {
                        ui.add(egui::Label::new("E(βγ) controls").strong());
                    });
                    ui.label("X axis");
                    ui.horizontal(|ui| {
                        ui.label("min: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.energy_min_x)
                                    .clamp_range(0. ..=chart_info.energy_max_x - 0.01),
                            )
                            .changed()
                            || recalculate_chart;
                        ui.label("max: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.energy_max_x)
                                    .clamp_range((chart_info.energy_min_x + 0.01)..=f64::INFINITY),
                            )
                            .changed()
                            || recalculate_chart;
                    });
                    ui.label("y axis");
                    ui.horizontal(|ui| {
                        ui.label("min: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.energy_min_y)
                                    .clamp_range(0. ..=chart_info.energy_max_x - 0.01),
                            )
                            .changed()
                            || recalculate_chart;
                        ui.label("max: ");
                        recalculate_chart = ui
                            .add(
                                egui::DragValue::new(&mut chart_info.energy_max_y)
                                    .clamp_range((chart_info.energy_min_y + 0.01)..=f64::INFINITY),
                            )
                            .changed()
                            || recalculate_chart;
                    });

                    ui.separator();

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.add(egui::Label::new("Author: Arkadiusz Żyłkowski"));
                    });
                });

            egui::CentralPanel::default().show(egui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.image(
                        bethe_texture_id,
                        Vec2::new(CHART_SIZE.0 as f32, CHART_SIZE.1 as f32),
                    );
                    ui.image(
                        energy_texture_id,
                        Vec2::new(CHART_SIZE.0 as f32, CHART_SIZE.1 as f32),
                    );
                });
            });

            // egui::SidePanel::right("right_panel").show(egui.ctx(), |ui| {

            // });

            if recalculate_chart {
                let (_, painter) = egui.ctx_and_painter_mut();
                draw_bethe(&chart_info, painter, bethe_texture_id);
                draw_energy(&chart_info, painter, energy_texture_id);
            }

            let (needs_repaint, shapes) = egui.end_frame(&display);
            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                let mut target = display.draw();
                egui.paint(&display, &mut target, shapes);

                target.finish().unwrap();
            }
        };

        match event {
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => {
                // println!("AAAAAAAAA");
                redraw()
            }
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                if egui.is_quit_event(&event) {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                }

                egui.on_event(&event);

                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }

            _ => (),
        }
    });
}
