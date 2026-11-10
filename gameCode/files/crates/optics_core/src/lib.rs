//! # Optics Core Engine
//! Handles physical camera optics, depth-of-field calculations, and exposure mathematics
//! for the tactical photography simulation loop.

#[derive(Debug, Clone)]
pub struct CameraOptics {
    pub focal_length_mm: f64,       // e.g., 24.0, 70.0, 300.0 mm
    pub f_number: f64,              // e.g., 1.8, 4.5, 22.0
    pub subject_distance_m: f64,    // Distance to targeted fauna in meters
    pub circle_of_confusion_mm: f64,// Standard full-frame reference (~0.03mm)
    pub iso: u32,                   // ISO sensitivity rating (100 - 6400)
    pub shutter_speed_s: f64,       // Shutter speed in seconds (e.g., 1.0 / 2000.0)
    pub ambient_base_ev: f64,       // Environmental lighting Exposure Value
}

#[derive(Debug, Clone, Copy)]
pub struct DepthOfFieldResult {
    pub near_limit_m: f64,
    pub far_limit_m: f64,
    pub total_dof_m: f64,
    pub hyperfocal_distance_m: f64,
}

impl CameraOptics {
    /// Creates a new camera configuration with standard 35mm full-frame defaults.
    pub fn new(focal_length_mm: f64, f_number: f64, subject_distance_m: f64) -> Self {
        Self {
            focal_length_mm,
            f_number,
            subject_distance_m,
            circle_of_confusion_mm: 0.03, // Full-frame standard CoC
            iso: 400,
            shutter_speed_s: 1.0 / 1000.0,
            ambient_base_ev: 12.0, // Overcast daylight baseline
        }
    }

    /// Calculates the hyperfocal distance ($H$) in meters.
    /// Formula: H = (f^2) / (N * c) + f
    pub fn hyperfocal_distance(&self) -> f64 {
        let f = self.focal_length_mm;
        let n = self.f_number;
        let c = self.circle_of_confusion_mm;
        
        // Convert mm result to meters
        let h_mm = (f.powi(2)) / (n * c) + f;
        h_mm / 1000.0
    }

    /// Computes Near Depth of Field, Far Depth of Field, and total operational window.
    pub fn calculate_dof(&self) -> DepthOfFieldResult {
        let s = self.subject_distance_m * 1000.0; // Convert to mm
        let f = self.focal_length_mm;
        let n = self.f_number;
        let c = self.circle_of_confusion_mm;

        let h = (f.powi(2)) / (n * c); // Hyperfocal distance in mm

        // Near limit
        let near_mm = (h * s) / (h + (s - f));
        
        // Far limit (handles asymptotic infinity edge cases)
        let far_mm = if s >= h {
            f64::INFINITY
        } else {
            (h * s) / (h - (s - f))
        };

        let near_m = near_mm / 1000.0;
        let far_m = if far_mm.is_infinite() { f64::INFINITY } else { far_mm / 1000.0 };
        let total_m = if far_m.is_infinite() { f64::INFINITY } else { far_m - near_m };

        DepthOfFieldResult {
            near_limit_m: near_m.max(0.0),
            far_limit_m: far_m,
            total_dof_m: total_m,
            hyperfocal_distance_m: self.hyperfocal_distance(),
        }
    }

    /// Evaluates current exposure against the environment's ambient EV.
    /// Returns EV offset (+/- stops of light).
    pub fn compute_exposure_delta(&self) -> f64 {
        // EV = log2(N^2 / t) - log2(ISO / 100)
        let aperture_term = (self.f_number.powi(2)) / self.shutter_speed_s;
        let iso_term = self.iso as f64 / 100.0;
        
        let current_ev = aperture_term.log2() - iso_term.log2();
        current_ev - self.ambient_base_ev
    }

    /// Checks if a moving target will exhibit motion blur based on shutter speed and angular velocity.
    pub fn check_motion_blur(&self, subject_velocity_mps: f64) -> bool {
        // Angular motion heuristic over exposure duration
        let angular_speed = subject_velocity_mps / self.subject_distance_m;
        let motion_spread_px = angular_speed * self.shutter_speed_s * 1200.0; // Scale factor
        
        motion_spread_px > 2.0 // Exceeding 2 pixels of linear displacement triggers blur penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperfocal_and_dof() {
        let optics = CameraOptics::new(300.0, 4.5, 15.0);
        let dof = optics.calculate_dof();
        
        assert!(dof.near_limit_m > 0.0);
        assert!(dof.far_limit_m > dof.near_limit_m);
        assert!(dof.hyperfocal_distance_m > 0.0);
    }
}
