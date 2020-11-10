use std::io::{self, Stdout, Write};

use crossterm::cursor::{self, MoveTo};
use crossterm::event::DisableMouseCapture;
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::QueueableCommand;
use crossterm::{execute, ExecutableCommand, Result};

use crate::{Pixel, Viewport};

// -----------------------------------------------------------------------------
//     - Raw mode -
// -----------------------------------------------------------------------------
fn raw_mode() -> Result<Stdout> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, DisableMouseCapture,)?;
    stdout.execute(cursor::Hide)?;
    stdout.execute(Clear(ClearType::All))?;
    Ok(stdout)
}

// -----------------------------------------------------------------------------
//     - Renderer -
// -----------------------------------------------------------------------------
/// Draws characters to a render target (most likely stdout)
pub struct Renderer<T: RenderTarget> {
    pub(crate) target: T,
}

impl<T: RenderTarget> Renderer<T> {
    /// Create a new target
    pub fn new(target: T) -> Self {
        Self { target }
    }

    /// Draw characters to screen
    pub fn render(&mut self, viewport: &mut Viewport) {
        self.target.render(viewport.pixels());
    }
}

// -----------------------------------------------------------------------------
//     - Render target-
// -----------------------------------------------------------------------------
/// Something that a render can render to.
pub trait RenderTarget {
    fn render(&mut self, pixels: Vec<Pixel>);
}

/// Render to stdout
pub struct StdoutTarget {
    stdout: Stdout,
}

impl StdoutTarget {
    /// Create a new stdout target.
    /// This sets stdout into raw mode.
    /// Once this is dropped it will disable raw mode.
    pub fn new() -> Result<Self> {
        let stdout = raw_mode()?;
        Ok(Self { stdout })
    }
}

impl RenderTarget for StdoutTarget {
    fn render(&mut self, pixels: Vec<Pixel>) {
        for (pixel, pos) in pixels {
            self.stdout
                .queue(MoveTo(pos.x, pos.y))
                .expect("failed to move cursor");
            self.stdout
                .queue(Print(pixel.to_string()))
                .expect("failed to print");
        }

        let _ = self.stdout.flush();
    }
}

impl Drop for StdoutTarget {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    fn camera() -> Camera {
        let pos = WorldPos::new(30, 30);
        let size = WorldSize::new(6, 6);
        Camera::new(pos, size)
    }

    fn viewport() -> Viewport {
        let pos = ScreenPos::new(2, 2);
        let size = ScreenSize::new(6, 6);
        Viewport::new(pos, size)
    }

    struct DummyTarget {
        pixels: Vec<Pixel>,
    }

    impl RenderTarget for DummyRender {
        fn render(&mut self, pixels: Vec<Pixel>) {
            self.pixels = pixels;
        }
    }

    #[test]
    fn render_pixels() {
        let cam = camera();
        let mut view = viewport();

        let min_x = cam.bounding_box.min_x();
        let min_y = cam.bounding_box.min_y();

        let a = ('A', WorldPos::new(min_x, min_y));
        let pixels = vec![a];

        view.draw(&cam, pixels);
        let mut renderer = Renderer::new(DummyTarget { pixels: Vec::new() });

        renderer.render(&mut view);

        let a = ('A', ScreenPos::new(2, 2)); // 2, 2 because of the viewport offset
        let pixels = vec![a];
        assert_eq!(pixels, renderer.target.pixels);
    }
}
