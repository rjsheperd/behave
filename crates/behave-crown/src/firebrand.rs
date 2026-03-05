//! Crown fire firebrand processor — determines the height and drift distance
//! of a firebrand from an active crown fire.
//!
//! Based on Frank Albini's "Program dist" for predicting spotting distance
//! from an active crown fire in uniformly forested flat terrain.
//!
//! All internal units are metric: canopy height (m), fireline intensity (kW/m),
//! wind speed (km/h), wind height (m), ember diameter (mm), distances (m).
//!
//! C++ source: CrownFirebrandProcessor.h / CrownFirebrandProcessor.cpp

/// Firebrand lofting and drift calculator for active crown fires.
///
/// Uses a lazy (dirty-flag) evaluation pattern: inputs are set via setters,
/// and calculations are only performed when an output is requested.
///
/// C++ class: `CrownFirebrandProcessor`
#[derive(Debug, Clone)]
pub struct CrownFirebrandProcessor {
    // Dirty flag
    is_dirty: bool,

    // Input properties
    canopy_ht: f64,    // Mean canopy height (m)
    delta_step: f64,   // Plume layer step size (m)
    ember_diam: f64,   // Viable firebrand diameter at surface (mm)
    fire_int: f64,     // Crown fireline intensity (kW/m)
    wind_ht: f64,      // Anemometer height (m)
    wind_speed: f64,   // Mean wind speed (km/h)

    // Primary outputs
    canopy_wind: f64,  // Wind speed at canopy top (m/s)
    flame_ht: f64,     // Mean flame height above canopy top (m)
    loft_ht: f64,      // Maximum firebrand loft height (m)
    spot_dist: f64,    // Maximum spotting distance (m)

    // Intermediate outputs (Albini variable names)
    ang: f64,          // Plume centerline angle above horizontal
    bf: f64,           // Plume width normal to centerline
    drift_x: f64,      // Normalized drift distance
    eta: f64,          // Safety factor for ember diameter (> 1)
    fm: f64,           // Mass flux in plume flow
    hf: f64,           // Normalized flame height (flame_ht / canopy_ht)
    layer: i32,        // Plume layer into which ember is lofted
    loft_x: f64,       // Normalized loft x-coordinate
    loft_z: f64,       // Normalized loft z-coordinate
    qfac: f64,         // Constant for drop()
    qreq: f64,         // Required q for ember_diam
    rhof: f64,         // Mean mass density of plume flow
    uc: f64,           // Normalized wind speed at canopy top
    uf: f64,           // Horizontal Favre average plume velocity
    vf: f64,           // Favre average plume velocity
    wf: f64,           // Vertical Favre average plume velocity
    wn: f64,           // Wind speed adjustment factor
}

impl CrownFirebrandProcessor {
    pub fn new(
        canopy_ht: f64,
        fire_int: f64,
        wind_speed: f64,
        wind_ht: f64,
        ember_diam: f64,
        delta_step: f64,
    ) -> Self {
        Self {
            is_dirty: true,
            canopy_ht,
            delta_step,
            ember_diam,
            fire_int,
            wind_ht,
            wind_speed,
            canopy_wind: 0.0,
            flame_ht: 0.0,
            loft_ht: 0.0,
            spot_dist: 0.0,
            ang: 0.0,
            bf: 0.0,
            drift_x: 0.0,
            eta: 1.0,
            fm: 0.0,
            hf: 0.0,
            layer: 0,
            loft_x: 0.0,
            loft_z: 0.0,
            qfac: 0.0,
            qreq: 0.0,
            rhof: 0.0,
            uc: 0.0,
            uf: 0.0,
            vf: 0.0,
            wf: 0.0,
            wn: 0.0,
        }
    }

    /// Convenience constructor with default step size (0.2).
    pub fn new_with_params(
        canopy_ht: f64,
        fire_int: f64,
        wind_speed: f64,
        wind_ht: f64,
        ember_diam: f64,
    ) -> Self {
        Self::new(canopy_ht, fire_int, wind_speed, wind_ht, ember_diam, 0.2)
    }

    /// Convenience constructor with default ember diameter (0.5 mm) and step size (0.2).
    pub fn with_defaults() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.5, 0.2)
    }

    // --- Primary output accessors ---

    pub fn firebrand_distance(&mut self) -> f64 {
        self.update();
        self.spot_dist
    }

    pub fn firebrand_height(&mut self) -> f64 {
        self.update();
        self.loft_ht
    }

    pub fn flame_height(&mut self) -> f64 {
        self.update();
        self.flame_ht
    }

    pub fn wind_speed_at_canopy(&mut self) -> f64 {
        self.update();
        self.canopy_wind
    }

    // --- Intermediate output accessors ---

    pub fn diameter_factor(&mut self) -> f64 {
        self.update();
        self.eta
    }

    pub fn loft_layer(&mut self) -> i32 {
        self.update();
        self.layer
    }

    pub fn normalized_drift_dist(&mut self) -> f64 {
        self.update();
        self.drift_x
    }

    pub fn normalized_flame_height(&mut self) -> f64 {
        self.update();
        self.hf
    }

    pub fn normalized_loft_distance(&mut self) -> f64 {
        self.update();
        self.loft_x
    }

    pub fn normalized_loft_height(&mut self) -> f64 {
        self.update();
        self.loft_z
    }

    pub fn normalized_wind_speed(&mut self) -> f64 {
        self.update();
        self.uc
    }

    pub fn plume_angle(&mut self) -> f64 {
        self.update();
        self.ang
    }

    // --- Input mutators ---

    pub fn set(
        &mut self,
        canopy_ht: f64,
        fire_int: f64,
        wind_speed: f64,
        wind_ht: f64,
        ember_diam: f64,
        delta_step: f64,
    ) {
        self.set_canopy_height(canopy_ht);
        self.set_delta_step(delta_step);
        self.set_ember_diameter(ember_diam);
        self.set_fireline_intensity(fire_int);
        self.set_wind_speed_height(wind_ht);
        self.set_wind_speed(wind_speed);
    }

    pub fn set_canopy_height(&mut self, canopy_ht: f64) {
        if canopy_ht != self.canopy_ht {
            self.canopy_ht = canopy_ht;
            self.is_dirty = true;
        }
    }

    pub fn set_delta_step(&mut self, delta_step: f64) {
        if delta_step != self.delta_step {
            self.delta_step = delta_step;
            self.is_dirty = true;
        }
    }

    pub fn set_ember_diameter(&mut self, ember_diam: f64) {
        if ember_diam != self.ember_diam {
            self.ember_diam = ember_diam;
            self.is_dirty = true;
        }
    }

    pub fn set_fireline_intensity(&mut self, fire_int: f64) {
        if fire_int != self.fire_int {
            self.fire_int = fire_int;
            self.is_dirty = true;
        }
    }

    pub fn set_wind_speed(&mut self, wind_speed: f64) {
        if wind_speed != self.wind_speed {
            self.wind_speed = wind_speed;
            self.is_dirty = true;
        }
    }

    pub fn set_wind_speed_height(&mut self, wind_ht: f64) {
        if wind_ht != self.wind_ht {
            self.wind_ht = wind_ht;
            self.is_dirty = true;
        }
    }

    // --- Private methods ---

    fn update(&mut self) {
        if self.is_dirty {
            self.process();
            self.is_dirty = false;
        }
    }

    fn process(&mut self) {
        self.reset();
        // Fireline intensity must be >= 1000 kW/m, and there must be canopy and wind
        if self.canopy_ht > 0.1 && self.wind_speed > 0.1 && self.fire_int >= 1000.0 {
            self.process_canopy_wind_speed();
            self.process_flame_height();
            self.process_flame_boundary_conditions();
            self.process_firebrand_loft();
            self.process_firebrand_drift();
        }
    }

    fn reset(&mut self) {
        self.canopy_wind = 0.0;
        self.flame_ht = 0.0;
        self.loft_ht = 0.0;
        self.spot_dist = 0.0;
        self.ang = 0.0;
        self.bf = 0.0;
        self.drift_x = 0.0;
        self.eta = 1.0;
        self.fm = 0.0;
        self.hf = 0.0;
        self.layer = 0;
        self.loft_x = 0.0;
        self.loft_z = 0.0;
        self.qfac = 0.0;
        self.qreq = 0.0;
        self.rhof = 0.0;
        self.uc = 0.0;
        self.uf = 0.0;
        self.vf = 0.0;
        self.wf = 0.0;
        self.wn = 0.0;
    }

    /// Adjusts input wind speed to wind speed at canopy top (m/s).
    /// From Albini's dist.for.
    fn process_canopy_wind_speed(&mut self) {
        let factor = if self.canopy_ht > 0.1 {
            1.0 + (1.0 + 2.94 * (self.wind_ht / self.canopy_ht)).ln()
        } else {
            1.0
        };
        self.canopy_wind = self.wind_speed / (3.6 * factor);
    }

    /// Calculate average flame height above canopy top (m).
    /// From Albini's dist.for function height().
    fn process_flame_height(&mut self) {
        let denom = self.canopy_ht * self.canopy_wind;
        if denom > 0.0 {
            let con = 7.791e-03 * self.fire_int / denom;
            let mut ylo = 1.0_f64;
            let mut yhi = con.exp();
            loop {
                let y = 0.5 * (ylo + yhi);
                let test = y * y.ln();
                if (test - con).abs() <= 1.0e-06 {
                    self.flame_ht = self.canopy_ht * (y - 1.0) / 2.94;
                    return;
                }
                if test >= con {
                    yhi = y;
                } else {
                    ylo = y;
                }
            }
        }
    }

    /// Calculate flame boundary conditions.
    /// From Albini's tip.for function tip().
    fn process_flame_boundary_conditions(&mut self) {
        self.hf = self.flame_ht / self.canopy_ht;
        self.wn = (9.82 * self.canopy_ht).sqrt();
        self.uc = self.canopy_wind / self.wn;

        self.qfac = 0.00838 / (self.uc * self.uc);

        let rfc = 1.0 + 2.94 * self.hf;
        let log_rfc = rfc.ln();

        self.fm = 0.468 * rfc * log_rfc;

        let fmuf = 1.3765 * (self.hf + rfc * log_rfc * log_rfc);
        self.uf = fmuf / self.fm;

        let ctn2f = rfc - 1.0 + rfc * log_rfc * log_rfc;
        let tang = 1.40 * self.hf / (self.uc * ctn2f.sqrt());
        self.ang = tang.atan();

        self.wf = tang * self.uf;
        self.vf = (self.uf * self.uf + self.wf * self.wf).sqrt();
        self.rhof = 0.6;
        self.bf = self.fm / (self.rhof * self.vf);
    }

    /// Determine firebrand loft height and position.
    /// From Albini's prof.for SUBROUTINE PROF() and drop.for SUBROUTINE DROP().
    fn process_firebrand_loft(&mut self) {
        let dlosmm = 0.064 * self.canopy_ht;
        let dtopmm = self.ember_diam + dlosmm;
        self.eta = dtopmm / dlosmm;

        let zc2 = self.hf;
        let xc2 = self.hf / self.ang.tan();
        let fmadd = 0.2735 * self.fm;
        let hfarg = 1.0 + 2.94 * self.hf;
        let fmuadd = 0.3765 * (self.hf + hfarg * hfarg.ln() * hfarg.ln());
        let dmwfac = 2.0 * self.fm / (3.0 * self.uc * self.uc);
        let tratf = 2.0 * self.fm / 3.0;

        let mut sing = self.ang.sin();
        let mut cosg = self.ang.cos();
        let mut delx = 0.5 * self.bf * sing;
        let mut delz = 0.5 * self.bf * cosg;
        let mut x = xc2;
        let mut z = self.hf;
        let mut v = self.vf;
        let mut w = self.wf;
        let mut fmw = self.fm * self.wf;

        let mut xb_lower = delx;
        let mut zb_lower = 0.0;
        let mut xb_upper = xc2 + delx;
        let mut zb_upper = zc2 - delz;

        let q_init = 0.5 * self.rhof * self.wf * self.wf;
        let mut q_upper = q_init;

        self.layer = 1;
        loop {
            self.qreq = self.qfac * (zb_upper + self.eta);
            if q_upper < self.qreq {
                self.loft_z = zb_lower;
                self.loft_x = xb_lower;
                self.loft_ht = self.canopy_ht * self.loft_z;
                return;
            }

            self.layer += 1;
            let dx = self.delta_step * cosg;
            let dz = self.delta_step * sing;
            x += dx;
            z += dz;
            let zarg = 1.0 + 2.94 * z;
            let fm = 0.34 * zarg * zarg.ln() + fmadd;
            let fmu = z + (0.34 * zarg * zarg.ln() * zarg.ln()) + fmuadd;
            let u = fmu / fm;
            fmw += (dmwfac / v) * dz;
            w = fmw / fm;
            v = (u * u + w * w).sqrt();
            let trat = 1.0 + tratf / fm;
            let b = fm * trat / v;
            sing = w / v;
            cosg = u / v;
            delx = 0.5 * b * sing;
            delz = 0.5 * b * cosg;
            q_upper = 0.5 * w * w / trat;
            xb_lower = xb_upper;
            xb_upper = x + delx;
            zb_lower = zb_upper;
            zb_upper = z - delz;

            if (self.layer as f64 * self.delta_step) > 10000.0 {
                return;
            }
        }
    }

    /// Determine drift distance of lofted firebrand.
    /// From Albini's drift.for SUBROUTINE DRIFT().
    fn process_firebrand_drift(&mut self) {
        let f0 = 1.0 + 2.94 * self.loft_z;
        let f1 = (self.eta / (self.eta + self.loft_z)).sqrt();
        let f2 = (self.eta / (self.eta - 0.34)).sqrt();
        let f3 = f2 / f1;
        let f2log = ((f2 + 1.0) / (f2 - 1.0)).ln();
        let f3log = ((f3 + 1.0) / (f3 - 1.0)).ln();
        let f = 1.0 + f0.ln() - f1 + (f3log - f2log) / f3;
        self.drift_x = 10.9 * f * self.uc * (self.loft_z + self.eta).sqrt();
        self.spot_dist = self.canopy_ht * (self.loft_x + self.drift_x);
    }
}

impl Default for CrownFirebrandProcessor {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.5, 0.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_inputs_zero_outputs() {
        let mut p = CrownFirebrandProcessor::with_defaults();
        assert_eq!(p.firebrand_distance(), 0.0);
        assert_eq!(p.firebrand_height(), 0.0);
        assert_eq!(p.flame_height(), 0.0);
    }

    #[test]
    fn below_threshold_intensity() {
        let mut p = CrownFirebrandProcessor::new(20.0, 999.0, 30.0, 10.0, 0.5, 0.2);
        assert_eq!(p.firebrand_distance(), 0.0);
    }

    #[test]
    fn active_crown_fire_produces_positive_distance() {
        let mut p = CrownFirebrandProcessor::new(20.0, 5000.0, 30.0, 10.0, 0.5, 0.2);
        let dist = p.firebrand_distance();
        assert!(dist > 0.0, "spot_dist should be positive, got {dist}");
        let ht = p.firebrand_height();
        assert!(ht > 0.0, "loft_ht should be positive, got {ht}");
    }

    #[test]
    fn higher_intensity_longer_distance() {
        let mut p1 = CrownFirebrandProcessor::new(20.0, 3000.0, 30.0, 10.0, 0.5, 0.2);
        let mut p2 = CrownFirebrandProcessor::new(20.0, 10000.0, 30.0, 10.0, 0.5, 0.2);
        assert!(
            p2.firebrand_distance() > p1.firebrand_distance(),
            "Higher intensity should produce longer spotting distance"
        );
    }

    #[test]
    fn dirty_flag_recalculates() {
        let mut p = CrownFirebrandProcessor::new(20.0, 5000.0, 30.0, 10.0, 0.5, 0.2);
        let d1 = p.firebrand_distance();
        p.set_fireline_intensity(10000.0);
        let d2 = p.firebrand_distance();
        assert!(d2 > d1, "After increasing intensity, distance should increase");
    }
}
