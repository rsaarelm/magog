use std::f32;
use cgmath::{SquareMatrix, Matrix2, Vector2};
use {Rect, Anchor};

/// Reversible affine 2D projection.
pub struct Projection {
    fwd: Matrix2<f32>,
    inv: Matrix2<f32>,
    offset: Vector2<f32>,
}

impl Projection {
    /// Construct a projection for given on-screen tile grid axes.
    ///
    /// Will return None if the axes specify a degenerate projection
    /// that can't be inverted.
    pub fn new<V: Into<[f32; 2]>>(x_axis: V, y_axis: V) -> Option<Projection> {
        let (x_axis, y_axis) = (x_axis.into(), y_axis.into());
        let fwd = Matrix2::from_cols(Vector2::from(x_axis),
                                     Vector2::from(y_axis));

        if let Some(inv) = fwd.invert() {
            Some(Projection {
                fwd: fwd,
                inv: inv,
                offset: Vector2::new(0.0, 0.0),
            })
        } else {
            // Degenerate matrix, no inverse found.
            None
        }
    }

    /// Add a view space offset to the projection.
    pub fn view_offset<V: Into<[f32; 2]>>(mut self, offset: V) -> Projection {
        self.offset = self.offset + Vector2::from(offset.into());
        self
    }

    /// Add a world space offset to the projection.
    pub fn world_offset<V: Into<[f32; 2]>>(self, offset: V) -> Projection {
        let offset = self.offset +
                     self.fwd.clone() * Vector2::from(offset.into());
        self
    }

    /// Project world space into screen space.
    pub fn project<V: Into<[f32; 2]>>(&self, world_pos: V) -> [f32; 2] {
        let v = self.fwd * Vector2::from(world_pos.into());
        (v + self.offset).into()
    }

    /// Project screen space into world space.
    pub fn inv_project<V: Into<[f32; 2]>>(&self, screen_pos: V) -> [f32; 2] {
        let translated = Vector2::from(screen_pos.into()) - self.offset;
        let v = self.inv * translated;
        v.into()
    }

    /// Return the world rectangle that perfectly covers the given
    /// screen rectangle.
    pub fn inv_project_rectangle(&self, screen_area: &Rect<f32>) -> Rect<f32> {
        unimplemented!();
        /*
        let mut mn = V2(f32::INFINITY, f32::INFINITY);
        let mut mx = V2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &sp in [Anchor::TopLeft,
                    Anchor::TopRight,
                    Anchor::BottomLeft,
                    Anchor::BottomRight]
                       .iter() {
            let wp = self.inv_project(screen_area.point(sp));
            mx = V2(mx.0.max(wp.0.ceil()), mx.1.max(wp.1.ceil()));
            mn = V2(mn.0.min(wp.0.floor()), mn.1.min(wp.1.floor()));
        }
        Rect(mn, mx - mn)
        */
    }
}

#[cfg(test)]
mod test {
    use Projection;
    use cgmath::vec2;

    fn verify<V>(proj: &Projection, world: V, screen: V)
        where V: Into<[f32; 2]> + Copy
    {
        assert_eq!(proj.project(world), screen.into());
        assert_eq!(proj.inv_project(screen), world.into());
        assert_eq!(proj.inv_project(proj.project(world)), world.into());
    }

    #[test]
    fn test_projection() {
        // Degenerate axes
        assert!(Projection::new(vec2(-10.0, 0.0), vec2(10.0, 0.0)).is_none());

        // Isometric projection
        let proj = Projection::new(vec2(16.0, 8.0), vec2(-16.0, 8.0))
                       .unwrap()
                       .view_offset(vec2(32.0, 16.0));
        verify(&proj, vec2(0.0, 0.0), vec2(32.0, 16.0));
        verify(&proj, vec2(1.0, 0.0), vec2(48.0, 24.0));
        verify(&proj, vec2(0.0, 1.0), vec2(16.0, 24.0));
        verify(&proj, vec2(0.5, 0.5), vec2(32.0, 24.0));
        verify(&proj, vec2(1.0, 1.0), vec2(32.0, 32.0));
    }
}
