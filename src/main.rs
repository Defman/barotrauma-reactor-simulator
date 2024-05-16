use std::{path::Path, time::Duration};

use anyhow::Result;
use barotrauma_simulator::{Controller, Input, Output, Reactor};
use plotters::{
    backend::BitMapBackend,
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    series::LineSeries,
    style::{full_palette::ORANGE, IntoFont, GREEN, RED, WHITE},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

struct Mesurements {
    temperature: Vec<f32>,
    fission: Vec<f32>,
    fission_target: Vec<f32>,
    fission_optimal: Vec<f32>,
    turbine: Vec<f32>,
    turbine_target: Vec<f32>,
    turbine_optimal: Vec<f32>,
}

impl Mesurements {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            temperature: Vec::with_capacity(capacity),
            fission: Vec::with_capacity(capacity),
            fission_target: Vec::with_capacity(capacity),
            fission_optimal: Vec::with_capacity(capacity),
            turbine: Vec::with_capacity(capacity),
            turbine_target: Vec::with_capacity(capacity),
            turbine_optimal: Vec::with_capacity(capacity),
        }
    }

    fn write_temperature_graph(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let root = BitMapBackend::new(path.as_ref(), (2048, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Temperature", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0..self.temperature.len() as u32, 0.0..10000.0f32)?;

        chart.configure_mesh().x_labels(10).y_labels(10).draw()?;

        chart.draw_series(LineSeries::new(
            (0..self.temperature.len() as u32).zip(std::iter::repeat(5000.0)),
            &ORANGE,
        ))?;

        chart.draw_series(LineSeries::new(
            (0..self.temperature.len() as u32).zip(self.temperature.iter().copied()),
            &RED,
        ))?;

        Ok(())
    }

    fn write_fission_graph(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let root = BitMapBackend::new(path.as_ref(), (2048, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Fission", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0..self.temperature.len() as u32, 0.0..100.0f32)?;

        chart.configure_mesh().x_labels(10).y_labels(10).draw()?;

        chart.draw_series(LineSeries::new(
            (0..self.fission_optimal.len() as u32).zip(self.fission_optimal.iter().copied()),
            &GREEN,
        ))?;

        chart.draw_series(LineSeries::new(
            (0..self.fission_target.len() as u32).zip(self.fission_target.iter().copied()),
            &ORANGE,
        ))?;

        chart.draw_series(LineSeries::new(
            (0..self.fission.len() as u32).zip(self.fission.iter().copied()),
            &RED,
        ))?;

        Ok(())
    }

    fn write_turbine_graph(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let root = BitMapBackend::new(path.as_ref(), (2048, 768)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Turbine", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0..self.temperature.len() as u32, 0.0..100.0f32)?;

        chart.configure_mesh().x_labels(10).y_labels(10).draw()?;

        chart.draw_series(LineSeries::new(
            (0..self.turbine.len() as u32).zip(self.turbine.iter().copied()),
            &RED,
        ))?;

        chart.draw_series(LineSeries::new(
            (0..self.turbine_target.len() as u32).zip(self.turbine_target.iter().copied()),
            &ORANGE,
        ))?;

        Ok(())
    }

    fn write_all_graphs(&self, path: impl AsRef<Path>) -> Result<()> {
        self.write_temperature_graph(path.as_ref().join("temperature.png"))?;
        self.write_fission_graph(path.as_ref().join("fission.png"))?;
        self.write_turbine_graph(path.as_ref().join("turbine.png"))?;

        Ok(())
    }
}

impl Controller for Mesurements {
    fn update(&mut self, output: &Output, input: &mut Input) {
        self.temperature.push(output.get_temperature());
        self.fission.push(output.get_fission_rate());
        self.fission_target.push(input.get_fission_rate());
        self.fission_optimal
            .push((input.get_turbine_rate() * 75.0) / output.get_fuel_potential());
        self.turbine.push(output.get_turbine_rate());
        self.turbine_target.push(input.get_turbine_rate());
    }
}

struct Simulation<C> {
    ticks: u64,
    reactor: Reactor,
    controller: C,
}

impl<C> Simulation<C> {
    pub fn new(duration: Duration, reactor: Reactor, controller: C) -> Self {
        let ticks = duration.as_secs() * 60;
        Self {
            ticks,
            reactor,
            controller,
        }
    }
}

impl<C> Simulation<C>
where
    C: Controller,
{
    fn run(mut self) -> C {
        for _ in 0..self.ticks {
            let (input, output) = self.reactor.controls();
            self.controller.update(&output, input);
            self.reactor.update(1.0 / 60.0);
        }
        self.controller
    }
}

struct SimpleController {
    a0: f32,
    a1: f32,
    a2: f32,
    prev_error: f32,
    prev_prev_error: f32,
    output: f32,
    estimated_temperature: f32,
}

impl SimpleController {
    fn new(kp: f32, ki: f32, kd: f32) -> Self {
        let a0 = kp + ki + kd;
        let a1 = -kp - 2.0 * kd;
        let a2 = kd;
        Self {
            a0,
            a1,
            a2,
            prev_error: 0.0,
            prev_prev_error: 0.0,
            output: 0.0,
            estimated_temperature: 0.0,
        }
    }
}

impl Controller for SimpleController {
    fn update(&mut self, output: &Output, input: &mut Input) {
        // if input.get_fission_rate() > 0.0 {
        //     self.estimated_temperature += 1000.0 / 60.0;
        // } else {
        //     self.estimated_temperature -= 1000.0 / 60.0;
        // }
        // let estimated = (input.get_turbine_rate() * 75.0) / output.get_fuel_potential();
        // let error = 5000.0 - output.get_temperature(); // + self.estimated_temperature;

        // self.output = self.output
        //     + self.a0 * error
        //     + self.a1 * self.prev_error
        //     + self.a2 * self.prev_prev_error;

        // self.prev_prev_error = self.prev_error;
        // self.prev_error = error;

        // let output = self.output;
        // input.set_fission_rate(output + estimated);

        if output.get_temperature() > 5000.0 {
            input.set_fission_rate(0.0);
        } else {
            input.set_fission_rate(100.0);
        }
    }
}

struct Load {
    tick: u64,
    min: f32,
    max: f32,
    periode: u64,
}

impl Load {
    fn new(min: f32, max: f32, periode: u64) -> Self {
        Self {
            tick: 0,
            min,
            max,
            periode,
        }
    }
}

impl Controller for Load {
    fn update(&mut self, _output: &Output, input: &mut Input) {
        self.tick = (self.tick + 1) % self.periode;

        if self.tick < self.periode / 2 {
            input.set_turbine_rate(self.max);
        } else {
            input.set_turbine_rate(self.min);
        }
    }
}

fn main() -> Result<()> {
    let path = Path::new("reactor");

    [80.0, 160.0, 240.0, 320.0]
        .par_iter()
        .copied()
        .try_for_each(|fuel_potential| -> Result<()> {
            let reactor = Reactor::new(fuel_potential, 4000.0);

            let mesurements = Mesurements::with_capacity(60 * 30);

            let load = Load::new(0.0, 100.0, 60 * 300);

            let simulation = Simulation::new(
                Duration::from_secs(60),
                reactor,
                (
                    load,
                    mesurements,
                    SimpleController::new(0.2 / 60.0, 0.00 / 60.0, 0.00 / 60.0),
                ),
            );
            let (_, messurements, _controller) = simulation.run();

            let max_temp = messurements
                .temperature
                .iter()
                .copied()
                .max_by(|a, b| a.total_cmp(b))
                .unwrap();

            if max_temp > 6482.0 {
                println!("Reactor is unsafe!");
            } else {
                println!("Reactor is safe!");
            }

            println!("max_temp: {}", max_temp);

            let path = path.join(format!("{fuel_potential}"));

            std::fs::create_dir_all(&path)?;
            messurements.write_all_graphs(&path)?;

            anyhow::Result::Ok(())
        })?;

    Ok(())
}
