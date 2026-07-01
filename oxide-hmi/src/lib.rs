use slint::{Model, VecModel};
use std::rc::Rc;
use embassy_executor::task;
use embassy_sync::channel::Channel;
use oxide_core::VehicleTelemetry;

slint::include_modules!();

#[task]
pub async fn ui_updater_task(
    channel: &'static Channel<embassy_sync::blocking_mutex::raw::NoopRawMutex, VehicleTelemetry, 16>,
) {
    let ui = AppWindow::new().unwrap();
    loop {
        let telemetry = channel.receive().await;
        let state = ui.global::<OxideVehicleState>();
        state.set_speed(telemetry.speed);
        state.set_rpm(telemetry.rpm as i32);
        state.set_soc(telemetry.soc as i32);

        if telemetry.temp > 100 {
            let leds = state.get_warning_leds();
            let mut new_leds = leds.as_any().downcast_ref::<VecModel<bool>>().unwrap().iter().collect::<Vec<_,_>>();
            new_leds[0] = true;
            state.set_warning_leds(Rc::new(VecModel::from(new_leds.to_vec())).into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_update() {
        // Slint UI logic is hard to test in a non-graphical environment.
    }
}