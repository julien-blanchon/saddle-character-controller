use saddle_character_controller_example_common as common;

use common::DemoConfig;

fn main() -> bevy::app::AppExit {
    common::run_demo(DemoConfig::water())
}
