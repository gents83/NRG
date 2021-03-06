use nrg_math::Vector4;
use nrg_messenger::Message;
use nrg_serialize::{Deserialize, Serialize};

use crate::{implement_widget_with_data, InternalWidget, Screen, WidgetData};

#[derive(Serialize, Deserialize)]
#[serde(crate = "nrg_serialize")]
pub struct Canvas {
    data: WidgetData,
}
implement_widget_with_data!(Canvas);

impl InternalWidget for Canvas {
    fn widget_init(&mut self) {
        if self.is_initialized() {
            return;
        }

        self.size(Screen::get_size())
            .selectable(false)
            .draggable(false)
            .style(WidgetStyle::Invisible);
    }

    fn widget_update(&mut self, _drawing_area_in_px: Vector4) {}

    fn widget_uninit(&mut self) {}
    fn widget_process_message(&mut self, _msg: &dyn Message) {}
    fn widget_on_layout_changed(&mut self) {}
}
