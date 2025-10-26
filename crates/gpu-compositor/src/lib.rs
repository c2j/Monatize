pub type TabId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceHandle {
    pub id: u64,
    pub width: u32,
    pub height: u32,
}

impl SurfaceHandle {
    pub fn new(id: u64, width: u32, height: u32) -> Self {
        Self { id, width, height }
    }
}

#[derive(Default)]
pub struct GpuCompositor {
    surfaces: std::collections::HashMap<TabId, SurfaceHandle>,
    frame_seq: u64,
}

impl GpuCompositor {
    pub fn new() -> Self { Self::default() }

    pub fn add_surface(&mut self, tab: TabId, handle: SurfaceHandle) {
        self.surfaces.insert(tab, handle);
    }

    pub fn remove_surface(&mut self, tab: TabId) -> Option<SurfaceHandle> {
        self.surfaces.remove(&tab)
    }

    /// Render one frame (stub). Returns number of active surfaces.
    pub fn render_frame(&mut self) -> anyhow::Result<usize> {
        self.frame_seq = self.frame_seq.wrapping_add(1);
        Ok(self.surfaces.len())
    }

    pub fn frames_rendered(&self) -> u64 { self.frame_seq }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_surface() {
        let mut comp = GpuCompositor::new();
        comp.add_surface(1, SurfaceHandle::new(42, 800, 600));
        assert_eq!(comp.render_frame().unwrap(), 1);
        assert_eq!(comp.frames_rendered(), 1);
        let removed = comp.remove_surface(1);
        assert!(removed.is_some());
        assert_eq!(comp.render_frame().unwrap(), 0);
    }

    #[test]
    fn remove_missing_is_none() {
        let mut comp = GpuCompositor::new();
        assert!(comp.remove_surface(999).is_none());
        assert_eq!(comp.render_frame().unwrap(), 0);
    }
}

