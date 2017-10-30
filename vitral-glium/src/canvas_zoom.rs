use euclid::{Point2D, Rect, Size2D};

/// Configuration for displaying a deferred rendering canvas in a window.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CanvasZoom {
    /// Fill the window, but preserve the aspect ratio of the canvas.
    AspectPreserving,
    /// Preserve the aspect ratio and only zoom to an integer multiple, keeping pixel areas equal.
    PixelPerfect,
}

impl CanvasZoom {
    /// Return a centered rectangle for the canvas in the given window.
    ///
    /// The rectangle will be given in OpenGL device coordinates, range -1.0 to 1.0.
    pub fn fit_canvas(self, window_size: Size2D<u32>, canvas_size: Size2D<u32>) -> Rect<f32> {
        match self {
            CanvasZoom::AspectPreserving => self.aspect_preserving(window_size, canvas_size),
            CanvasZoom::PixelPerfect => self.pixel_perfect(window_size, canvas_size),
        }
    }

    fn with_scale(
        self,
        scale: f32,
        window_size: Size2D<u32>,
        canvas_size: Size2D<u32>,
    ) -> Rect<f32> {
        let dim = Size2D::new(
            (scale * canvas_size.width as f32) * 2.0 / window_size.width as f32,
            (scale * canvas_size.height as f32) * 2.0 / window_size.height as f32,
        );
        let offset = Point2D::new(-dim.width / 2.0, -dim.height / 2.0);
        Rect::new(offset, dim)
    }

    fn aspect_preserving(self, window_size: Size2D<u32>, canvas_size: Size2D<u32>) -> Rect<f32> {
        // Scale based on whichever of X or Y axis is the tighter fit.
        let scale = (window_size.width as f32 / canvas_size.width as f32).min(
            window_size.height as f32 / canvas_size.height as f32,
        );

        self.with_scale(scale, window_size, canvas_size)
    }

    fn pixel_perfect(self, window_size: Size2D<u32>, canvas_size: Size2D<u32>) -> Rect<f32> {
        // Clip window dimensions to even numbers, pixel-perfect rendering has artifacts with odd
        // window dimensions.
        let window_size = Size2D::new(window_size.width & !1, window_size.height & !1);

        // Scale based on whichever of X or Y axis is the tighter fit.
        let mut scale = (window_size.width as f32 / canvas_size.width as f32).min(
            window_size.height as f32 / canvas_size.height as f32,
        );

        if scale > 1.0 {
            // Snap to pixel scale if more than 1 window pixel per canvas pixel.
            scale = scale.floor();
        }

        self.with_scale(scale, window_size, canvas_size)
    }

    /// Map physical window coordinates to logical canvas coordinates.
    pub fn screen_to_canvas(
        self,
        mut window_size: Size2D<u32>,
        canvas_size: Size2D<u32>,
        screen_pos: Point2D<f32>,
    ) -> Point2D<f32> {
        let rect = self.fit_canvas(window_size, canvas_size);
        let rp = rect.origin.to_untyped();
        let rs = rect.size.to_untyped();

        if self == CanvasZoom::PixelPerfect {
            window_size = Size2D::new(window_size.width & !1, window_size.height & !1);
        }

        // Transform to device coordinates.
        let sx = screen_pos.x * 2.0 / window_size.width as f32 - 1.0;
        let sy = screen_pos.y * 2.0 / window_size.height as f32 - 1.0;

        Point2D::new(
            (sx - rp.x) * canvas_size.width as f32 / rs.width,
            (sy - rp.y) * canvas_size.height as f32 / rs.height,
        )
    }
}
