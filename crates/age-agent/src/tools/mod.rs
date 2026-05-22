pub mod level;
pub mod ui;

use age_core::tool::registry::ToolRegistry;

pub fn register_dev_tools(registry: &mut ToolRegistry) {
    level::register_level_tools(registry);
    ui::register_ui_tools(registry);
}
