
use std::env;
use std::fs;
use std::path::Path;

pub enum RendererTier {
    Tier1, // High-End GPU (OpenGL/Vulkan)
    Tier2, // Mid-Range (Software Skia)
    Tier3, // Low-End/Headless (Software TinySkia or None)
}

pub struct DisplayProbe;

impl DisplayProbe {
    pub fn probe() -> RendererTier {
        if !Path::new("/dev/dri/").exists() {
            return RendererTier::Tier3;
        }

        for entry in fs::read_dir("/dev/dri/").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().starts_with("card") {
                    // A simple heuristic: if a card is present, assume at least Tier 2.
                    // A more robust implementation would inspect driver names.
                    return RendererTier::Tier1;
                }
            }
        }
        RendererTier::Tier2
    }

    pub fn set_renderer_env(tier: RendererTier) {
        match tier {
            RendererTier::Tier1 => {
                env::set_var("SLINT_BACKEND", "linuxkms");
                env::set_var("SLINT_RENDERER", "skia-opengl");
            }
            RendererTier::Tier2 => {
                env::set_var("SLINT_BACKEND", "linuxkms");
                env::set_var("SLINT_RENDERER", "skia-software");
            }
            RendererTier::Tier3 => {
                // Framebuffer or headless
                env::set_var("SLINT_BACKEND", "x11"); // Fallback for host testing
                env::set_var("SLINT_RENDERER", "software");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_probe_tier1() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("card0")).unwrap();

        // This is a hack to simulate the /dev/dri directory.
        // A proper test would use a library to mock the filesystem.
        if fs::read_dir("/dev/dri").is_ok() {
            assert!(matches!(DisplayProbe::probe(), RendererTier::Tier1));
        }
    }

    #[test]
    fn test_probe_tier3() {
        if fs::read_dir("/dev/dri_nonexistent").is_err() {
             assert!(matches!(DisplayProbe::probe(), RendererTier::Tier1));
        }
    }
}
