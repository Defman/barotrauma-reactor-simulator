use std::fmt::Debug;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Input {
    fission_rate: f32,
    turbine_rate: f32,
    load: f32,
}

impl Input {
    fn new() -> Self {
        Self {
            fission_rate: 0.0,
            turbine_rate: 0.0,
            load: 0.0,
        }
    }

    pub fn set_fission_rate(&mut self, fission_rate: f32) {
        self.fission_rate = fission_rate.clamp(0.0, 100.0);
    }

    pub fn get_fission_rate(&self) -> f32 {
        self.fission_rate
    }

    pub fn set_turbine_rate(&mut self, turbine_rate: f32) {
        self.turbine_rate = turbine_rate.clamp(0.0, 100.0);
    }

    pub fn get_turbine_rate(&self) -> f32 {
        self.turbine_rate
    }

    pub fn set_load(&mut self, load: f32) {
        self.load = load.clamp(0.0, 100.0);
    }

    pub fn get_load(&self) -> f32 {
        self.load
    }
}

#[derive(Serialize)]
pub struct Output {
    temperature: f32,
    load: f32,
    power: f32,
    fuel_potential: f32,
    fission_rate: f32,
    turbine_rate: f32,
}

impl Output {
    fn new() -> Self {
        Self {
            temperature: 0.0,
            load: 0.0,
            power: 0.0,
            fuel_potential: 0.0,
            fission_rate: 0.0,
            turbine_rate: 0.0,
        }
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }

    pub fn get_load(&self) -> f32 {
        self.load
    }

    pub fn get_power(&self) -> f32 {
        self.power
    }

    pub fn get_fuel_potential(&self) -> f32 {
        self.fuel_potential
    }

    /// Hidden, cannot read ingame
    pub fn get_fission_rate(&self) -> f32 {
        self.fission_rate
    }

    /// Hidden, cannot read ingame
    pub fn get_turbine_rate(&self) -> f32 {
        self.turbine_rate
    }
}

pub struct Reactor {
    fuel_potential: f32,
    power_max: f32,
    turbine: Turbine,
    core: Core,
    load: f32,
    input: Input,
    temperature: f32,
    output: Output,
}

impl Debug for Reactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reactor")
            .field("turbine", &self.turbine.value)
            .field("turbine_target", &self.turbine.target)
            .field("fission", &self.core.value)
            .field("fission_target", &self.core.target)
            .field("fuel_potential", &self.fuel_potential)
            .field("power_max", &self.power_max)
            .field("load", &self.load)
            .field("temperatur", &self.temperature)
            .finish()
    }
}

impl Reactor {

    pub fn new(fuel_potential: f32, power_max: f32) -> Self {
        Self {
            input: Input::new(),
            core: Core::new(),
            turbine: Turbine::new(),
            power_max,
            fuel_potential,
            load: 0.0,
            temperature: 0.0,
            output: Output::new(),
        }
    }

    pub fn get_output(&self) -> &Output {
        &self.output
    }

    pub fn get_input(&self) -> &Input {
        &self.input
    }

    pub fn get_input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn controls(&mut self) -> (&mut Input, &Output) {
        (&mut self.input, &self.output)
    }
}

struct Core {
    value: f32,
    target: f32,
}

impl Core {
    fn new() -> Self {
        Self {
            value: 0.0,
            target: 0.0,
        }
    }

    fn update(&mut self, new_target: f32, time_delta: f32) {
        self.target = if self.target >= new_target {
            (self.target - time_delta * 5.0).max(new_target)
        } else {
            (self.target + time_delta * 5.0).min(new_target)
        };
        let heat_potential = 320.0;

        self.value += (self.target.min(heat_potential) - self.value) * time_delta;
        self.value = self.value.clamp(0.0, 100.0);
    }
}

struct Turbine {
    value: f32,
    target: f32,
}

impl Turbine {
    fn new() -> Self {
        Self {
            value: 0.0,
            target: 0.0,
        }
    }

    fn update(&mut self, new_target: f32, time_delta: f32) {
        self.target = if self.target >= new_target {
            (self.target - time_delta * 5.0).max(new_target)
        } else {
            (self.target + time_delta * 5.0).min(new_target)
        };
        self.value += (self.target - self.value) * time_delta;
        self.value = self.value.clamp(0.0, 100.0);
    }
}

impl Reactor {
    pub fn update(&mut self, time_delta: f32) {
        self.update_temperatur(time_delta);
        
        // self.core.target = self.input.fission_rate;
        self.core.update(self.input.fission_rate, time_delta);

        // self.turbine.target = self.input.turbine_rate;
        self.turbine.update(self.input.turbine_rate, time_delta);

        // Update outputs
        self.output.fuel_potential = self.fuel_potential;
        self.output.fission_rate = self.get_fission_rate();
        self.output.load = self.input.get_load();
        self.output.turbine_rate = self.get_turbine_rate();
    }

    fn update_temperatur(&mut self, time_delta: f32) {
        let heat_supply = self.heat_supply();

        let temperatur_delta = (heat_supply - self.turbine.value * 100.0) - self.temperature;
        
        self.temperature += (temperatur_delta.signum() * 1000.0 * time_delta).clamp(-temperatur_delta.abs(), temperatur_delta.abs());
        self.temperature = self.temperature.clamp(0.0, 10000.0);

        self.output.temperature = self.temperature;
    }

    pub fn heat_demand(&self) -> f32 {
        self.turbine.value * 75.0
    }

    pub fn heat_supply(&self) -> f32 {
        2.0 * self.core.value * self.fuel_potential
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }

    pub fn get_fission_rate(&self) -> f32 {
        self.core.value
    }

    pub fn get_turbine_rate(&self) -> f32 {
        self.turbine.value
    }

    pub fn set_fission_rate(&mut self, fission_rate: f32) {
        self.input.fission_rate = fission_rate.clamp(0.0, 100.0);
    }

    pub fn set_turbine_rate(&mut self, turbine_rate: f32) {
        self.input.turbine_rate = turbine_rate.clamp(0.0, 100.0);
    }

    pub fn set_load(&mut self, load: f32) {
        self.load = load.max(0.0);
    }

    pub fn get_power(&self) -> f32 {
        self.turbine.value * self.power_max / 100.0
    }
}