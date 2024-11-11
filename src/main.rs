use eframe::egui;
use egui::ViewportBuilder;
use resvg::{render, tiny_skia};
use std::fs::File;
use std::io::{BufReader, Read};
use tiny_skia::Pixmap;
use usvg::{Options, Tree};

struct SvgConverterApp {
    input_path: String,
    output_path: String,
    scale: u32,
    status_message: String,
    original_width: Option<u32>,
    original_height: Option<u32>,
    will_be_width: Option<u32>,
    will_be_height: Option<u32>,
}

impl Default for SvgConverterApp {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_path: String::from("output.png"),
            scale: 1,
            status_message: String::new(),
            original_width: None,
            original_height: None,
            will_be_width: None,
            will_be_height: None,
        }
    }
}

impl SvgConverterApp {
    fn update_dimensions(&mut self) {
        if self.input_path.is_empty() {
            self.original_width = None;
            self.original_height = None;
        } else {
            if let Ok(file) = File::open(&self.input_path) {
                let mut reader = BufReader::new(file);
                let mut svg_data = String::new();
                if reader.read_to_string(&mut svg_data).is_ok() {
                    let options = Options::default();
                    if let Ok(rtree) = Tree::from_str(&svg_data, &options) {
                        self.original_width = Some(rtree.size.width() as u32);
                        self.original_height = Some(rtree.size.height() as u32);
                    }
                }
            }
        }

        // Always update the will_be dimensions
        self.will_be_width = self.original_width.map(|w| w * self.scale);
        self.will_be_height = self.original_height.map(|h| h * self.scale);
    }

    fn svg_to_png(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Read SVG file
        let mut svg_data = String::new();
        let file = File::open(&self.input_path)?;
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut svg_data)?;

        // Parse SVG
        let options = Options::default();
        let rtree = Tree::from_str(&svg_data, &options)?;

        // Calculate dimensions
        let width = (rtree.size.width() as u32) * self.scale;
        let height = (rtree.size.height() as u32) * self.scale;

        // Create a pixel map with scaled dimensions
        let mut pixmap = Pixmap::new(width, height).ok_or("Failed to create pixmap")?;

        // Apply transform for scaling
        let transform = tiny_skia::Transform::from_scale(self.scale as f32, self.scale as f32);

        // Render SVG to pixel map
        render(&rtree, usvg::FitTo::Original, transform, pixmap.as_mut());

        // Save to PNG file
        pixmap.save_png(&self.output_path)?;

        #[cfg(target_os = "macos")]
        std::process::Command::new("open")
            .arg(&self.output_path)
            .spawn()?;

        Ok(())
    }
}

impl eframe::App for SvgConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_dimensions(); // Call this every frame

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

                // Scaling factor dropdown
                ui.add_space(10.0);
                let mut scale_changed = false;
                ui.horizontal(|ui| {
                    ui.label("Scale:");
                    scale_changed |= egui::ComboBox::from_label("Select Scale")
                        .selected_text(format!("{}", self.scale))
                        .show_ui(ui, |ui| {
                            for &scale in &[1, 2, 4, 8, 16, 32, 64] {
                                ui.selectable_value(&mut self.scale, scale, format!("{}x", scale));
                            }
                        })
                        .response
                        .changed();
                });

                ui.add_space(10.0);

                if let (Some(original_width), Some(original_height)) =
                    (self.original_width, self.original_height)
                {
                    ui.label(format!(
                        "Original size: {}x{}",
                        original_width, original_height
                    ));
                }

                if let (Some(will_be_width), Some(will_be_height)) =
                    (self.will_be_width, self.will_be_height)
                {
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
