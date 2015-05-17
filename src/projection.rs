use std::f32;
use nalgebra::{inv, Mat2, Vec2};
use geom::{V2, Rect};
use {Anchor};

/// Reversible affine 2D projection.
pub struct Projection {
    fwd: Mat2<f32>,
    inv: Mat2<f32>,
    offset: V2<f32>,
}

impl Projection {
    /// Construct a projection for given on-screen tile grid axes.
    ///
    /// Will return None if the axes specify a degenerate projection
    /// that can't be inverted.
    pub fn new(x_axis: V2<f32>, y_axis: V2<f32>) -> Option<Projection> {
        let fwd = Mat2::new(x_axis.0, y_axis.0, x_axis.1, y_axis.1);

        if let Some(inv) = inv(&fwd) {
            Some(Projection {
                fwd: fwd,
                inv: inv,
                offset: V2(0.0, 0.0),
            })
        } else {
            // Degenerate matrix, no inverse found.
            None
        }
    }

    /// Add a view space offset to the projection.
    pub fn view_offset(mut self, offset: V2<f32>) -> Projection {
        self.offset = self.offset + offset;
        self
    }

    /// Add a world space offset to the projection.
    pub fn world_offset(self, offset: V2<f32>) -> Projection {
        let view_offset = self.fwd * Vec2::new(offset.0, offset.1);
        self.view_offset(V2(view_offset.x, view_offset.y))
    }

    /// Project world space into screen space.
    pub fn project(&self, world_pos: V2<f32>) -> V2<f32> {
        let v = self.fwd * Vec2::new(world_pos.0, world_pos.1);
        V2(v.x + self.offset.0, v.y + self.offset.1)
    }

    /// Project screen space into world space.
    pub fn inv_project(&self, screen_pos: V2<f32>) -> V2<f32> {
        let translated = screen_pos - self.offset;
        let v = self.inv * Vec2::new(translated.0, translated.1);
        V2(v.x, v.y)
    }

    /// Return the world rectangle that perfectly covers the given
    /// screen rectangle.
    pub fn inv_project_rectangle(&self, screen_area: &Rect<f32>) -> Rect<f32> {
        let mut mn = V2(f32::INFINITY, f32::INFINITY);
        let mut mx = V2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &sp in [Anchor::TopLeft, Anchor::TopRight, Anchor::BottomLeft, Anchor::BottomRight].iter() {
            let wp = self.inv_project(screen_area.point(sp));
            mx = V2(mx.0.max(wp.0.ceil()), mx.1.max(wp.1.ceil()));
            mn = V2(mn.0.min(wp.0.floor()), mn.1.min(wp.1.floor()));
        }
        Rect(mn, mx - mn)
    }
}

#[cfg(test)]
mod test {
    use ::geom::{V2};
    use ::{Projection};

    fn verify(proj: &Projection, world: V2<f32>, screen: V2<f32>) {
        assert_eq!(proj.project(world), screen);
        assert_eq!(proj.inv_project(screen), world);
        assert_eq!(proj.inv_project(proj.project(world)), world);
    }
    #[test]
    fn test_projection(){
        // Degenerate axes
        assert!(Projection::new(V2(-10.0, 0.0), V2(10.0, 0.0)).is_none());

        // Isometric projection
        let proj = Projection::new(V2(16.0, 8.0), V2(-16.0, 8.0))
            .unwrap().view_offset(V2(32.0, 16.0));
        verify(&proj, V2(0.0, 0.0), V2(32.0, 16.0));
        verify(&proj, V2(1.0, 0.0), V2(48.0, 24.0));
        verify(&proj, V2(0.0, 1.0), V2(16.0, 24.0));
        verify(&proj, V2(0.5, 0.5), V2(32.0, 24.0));
        verify(&proj, V2(1.0, 1.0), V2(32.0, 32.0));
    }
}
