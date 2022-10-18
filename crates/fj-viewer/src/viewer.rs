use fj_interop::{
    processed_shape::ProcessedShape, status_report::StatusReport,
};
use fj_math::Aabb;
use tracing::warn;

use crate::{
    gui::Gui, Camera, DrawConfig, InputHandler, Renderer, RendererInitError,
    Screen,
};

/// The Fornjot model viewer
pub struct Viewer {
    /// The camera
    pub camera: Camera,

    /// The draw config
    pub draw_config: DrawConfig,

    /// The GUI
    pub gui: Gui,

    /// The input handler
    pub input_handler: InputHandler,

    /// The renderer
    pub renderer: Renderer,

    /// The shape
    pub shape: Option<ProcessedShape>,
}

impl Viewer {
    /// Construct a new instance of `Viewer`
    pub async fn new(screen: &impl Screen) -> Result<Self, RendererInitError> {
        let renderer = Renderer::new(screen).await?;
        let gui = renderer.init_gui();

        Ok(Self {
            camera: Camera::default(),
            draw_config: DrawConfig::default(),
            gui,
            input_handler: InputHandler::default(),
            renderer,
            shape: None,
        })
    }

    /// Toggle the "draw model" setting
    pub fn toggle_draw_model(&mut self) {
        self.draw_config.draw_model = !self.draw_config.draw_model
    }

    /// Toggle the "draw mesh" setting
    pub fn toggle_draw_mesh(&mut self) {
        if self.renderer.is_line_drawing_available() {
            self.draw_config.draw_mesh = !self.draw_config.draw_mesh
        }
    }

    /// Toggle the "draw debug" setting
    pub fn toggle_draw_debug(&mut self) {
        if self.renderer.is_line_drawing_available() {
            self.draw_config.draw_debug = !self.draw_config.draw_debug
        }
    }

    /// Handle the shape being updated
    pub fn handle_shape_update(&mut self, shape: ProcessedShape) {
        self.renderer
            .update_geometry((&shape.mesh).into(), (&shape.debug_info).into());
        self.camera.update_planes(&shape.aabb);

        self.shape = Some(shape);
    }

    /// Draw the graphics
    pub fn draw(
        &mut self,
        scale_factor: f32,
        status: &mut StatusReport,
        egui_input: egui::RawInput,
    ) {
        let aabb = self
            .shape
            .as_ref()
            .map(|shape| shape.aabb)
            .unwrap_or_else(Aabb::default);

        self.gui.update(
            egui_input,
            &mut self.draw_config,
            &aabb,
            status,
            self.renderer.is_line_drawing_available(),
        );

        if let Err(err) = self.renderer.draw(
            &self.camera,
            &self.draw_config,
            scale_factor,
            &mut self.gui,
        ) {
            warn!("Draw error: {}", err);
        }
    }
}
