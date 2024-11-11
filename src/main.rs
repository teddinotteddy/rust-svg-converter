use eframe::egui;
use egui::ViewportBuilder;
use resvg::{render, tiny_skia};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use tiny_skia::Pixmap;
use usvg::{Options, Tree};

struct SvgConverterApp {
    input_path: String,
    output_path: String,
    scale: u32,
    status_message: String,
    original_dimensions: Option<(u32, u32)>,
    scaled_dimensions: Option<(u32, u32)>,
}

impl Default for SvgConverterApp {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: String::from("output.png"),
            scale: 1,
            status_message: String::new(),
            original_dimensions: None,
            scaled_dimensions: None,
        }
    }
}

impl SvgConverterApp {
    fn update_dimensions(&mut self) {
        self.original_dimensions = if self.input_path.is_empty() {
            None
        } else {
            File::open(&self.input_path)
                .ok()
                .and_then(|file| {
                    let mut svg_data = String::new();
                    let mut reader = std::io::BufReader::new(file);
                    reader.read_to_string(&mut svg_data).ok()?;
                    Tree::from_str(&svg_data, &Options::default()).ok()
                })
                .map(|rtree| (rtree.size.width() as u32, rtree.size.height() as u32))
        };

        self.scaled_dimensions = self
            .original_dimensions
            .map(|(w, h)| (w * self.scale, h * self.scale));
    }

    fn svg_to_png(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut svg_data = String::new();
        File::open(&self.input_path)?.read_to_string(&mut svg_data)?;

        let rtree = Tree::from_str(&svg_data, &Options::default())?;
        let (width, height) = self.scaled_dimensions.ok_or("Dimensions not calculated")?;

        let mut pixmap = Pixmap::new(width, height).ok_or("Failed to create pixmap")?;
        let transform = tiny_skia::Transform::from_scale(self.scale as f32, self.scale as f32);

        render(&rtree, usvg::FitTo::Original, transform, pixmap.as_mut());
        pixmap.save_png(&self.output_path)?;

        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(&self.output_path).spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/C", "start", "", &self.output_path])
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open").arg(&self.output_path).spawn()?;
        }

        Ok(())
    }
}

impl eframe::App for SvgConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_dimensions();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("SVG to PNG Converter");
                ui.add_space(10.0);

                let mut input_changed = false;
                ui.horizontal(|ui| {
                    ui.label("Input SVG:");
                    input_changed |= ui.text_edit_singleline(&mut self.input_path).changed();
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("SVG files", &["svg"])
                            .pick_file()
                        {
                            self.input_path = path.display().to_string();
                            input_changed = true;
                        }
                    }
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Output PNG:");
                    ui.text_edit_singleline(&mut self.output_path);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PNG files", &["png"])
                            .save_file()
                        {
                            self.output_path = path.display().to_string();
                        }
                    }
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    egui::ComboBox::from_label("Select Scale")
                        .selected_text(self.scale.to_string())
                        .show_ui(ui, |ui| {
                            for &scale in &[1, 2, 4, 8, 16, 32, 64] {
                                ui.selectable_value(&mut self.scale, scale, format!("{}x", scale));
                            }
                        });
                });

                ui.add_space(10.0);

                if let Some((original_width, original_height)) = self.original_dimensions {
                    ui.label(format!(
                        "Original size: {}x{}",
                        original_width, original_height
                    ));
                }

                if let Some((will_be_width, will_be_height)) = self.scaled_dimensions {
                    ui.label(format!(
                        "Will be size: {}x{}",
                        will_be_width, will_be_height
                    ));
                }

                ui.add_space(10.0);

                if ui.button("Convert").clicked() {
                    match self.svg_to_png() {
                        Ok(()) => {
                            self.status_message = format!(
                                "Successfully converted {} to {}",
                                self.input_path, self.output_path
                            );
                        }
                        Err(e) => {
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                }

                ui.add_space(5.0);
                ui.label(&self.status_message);
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_min_inner_size([480.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SVG to PNG Converter",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "Geist".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/Geist-Regular.ttf")),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "Geist".to_owned());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "Geist".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Box::new(SvgConverterApp::default())
        }),
    )
}
