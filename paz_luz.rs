use ash::vk;

pub struct ColorPipeline {
    pub surface_format: vk::SurfaceFormatKHR,
    pub color_space: vk::ColorSpaceKHR,
}

impl ColorPipeline {
    pub fn init_hdr() -> Self {
        // Buscamos soporte para BT2020_HDR10 o scRGB
        Self {
            surface_format: vk::SurfaceFormatKHR {
                format: vk::Format::A2B10G10R10_UNORM_PACK32,
                color_space: vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT,
            },
            color_space: vk::ColorSpaceKHR::HDR10_ST2084_EXT,
        }
    }
}
