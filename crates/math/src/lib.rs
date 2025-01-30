//! This crate contains generic and handler specific calculation functions used
//! during packet and server processing. It contains calculations for screen
//! updating, attacking, exploit checking, etc.

#![cfg_attr(not(feature = "std"), no_std)]

pub const SCREEN_DISTANCE: u16 = 18;
pub const RADIAN_TO_DEGREE: f32 = 57.29;
pub const MAX_DIFFERENCE_IN_ELEVATION: u16 = 210;

fn atan2(y: f32, x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        y.atan2(x)
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::atan2f(y, x)
    }
    #[cfg(not(any(feature = "std", feature = "libm")))]
    {
        compile_error!("Either the `std` or `libm` feature must be enabled to use `atan2`")
    }
}

fn abs(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.abs()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::fabsf(x)
    }
    #[cfg(not(any(feature = "std", feature = "libm")))]
    {
        compile_error!("Either the `std` or `libm` feature must be enabled to use `abs`")
    }
}

fn pow(x: f32, y: i32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.powi(y)
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::powf(x, y as f32)
    }
    #[cfg(not(any(feature = "std", feature = "libm")))]
    {
        compile_error!("Either the `std` or `libm` feature must be enabled to use `powi`")
    }
}

fn sqrt(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.sqrt()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::sqrtf(x)
    }
    #[cfg(not(any(feature = "std", feature = "libm")))]
    {
        compile_error!("Either the `std` or `libm` feature must be enabled to use `sqrt`")
    }
}

fn round(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.round()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::roundf(x)
    }
    #[cfg(not(any(feature = "std", feature = "libm")))]
    {
        compile_error!("Either the `std` or `libm` feature must be enabled to use `round`")
    }
}

/// This function returns true if an object is within the bounds of another
/// object's screen.
pub fn in_screen(p1: (u16, u16), p2: (u16, u16)) -> bool {
    in_range(p1, p2, SCREEN_DISTANCE)
}

/// This function returns true if an object is within the range.
pub fn in_range(p1: (u16, u16), p2: (u16, u16), range: u16) -> bool {
    let (delta_x, delta_y) = delta(p1, p2);
    delta_x <= range && delta_y <= range
}

/// This function returns delta (x, y).
pub fn delta(p1: (u16, u16), p2: (u16, u16)) -> (u16, u16) {
    let x1 = p1.0 as i16;
    let x2 = p2.0 as i16;
    let y1 = p1.1 as i16;
    let y2 = p2.1 as i16;
    let delta_x = (x2 - x1).abs();
    let delta_y = (y2 - y1).abs();
    (delta_x as u16, delta_y as u16)
}

/// This function checks the elevation difference of two tiles.
pub fn within_elevation(new: u16, initial: u16) -> bool {
    let new = new as i16;
    let initial = initial as i16;
    (new - initial) < MAX_DIFFERENCE_IN_ELEVATION as i16
}

/// This function returns the angle for a jump or attack.
pub fn get_angle(p1: (u16, u16), p2: (u16, u16)) -> f32 {
    let (delta_x, delta_y) = delta(p1, p2);
    let delta_x = delta_x as f32;
    let delta_y = delta_y as f32;
    let angle = (atan2(delta_y, delta_x) * RADIAN_TO_DEGREE) + 90.0;
    if angle.is_sign_negative() {
        270.0 + (90.0 - abs(angle))
    } else {
        angle
    }
}

/// This function returns the distance between two objects.
pub fn get_distance(p1: (u16, u16), p2: (u16, u16)) -> f32 {
    let x1 = p1.0 as f32;
    let x2 = p2.0 as f32;
    let y1 = p1.1 as f32;
    let y2 = p2.1 as f32;
    sqrt(pow(x2 - x1, 2) + pow(y2 - y1, 2))
}

/// This function returns the direction for a jump or attack.
pub fn get_direction_sector(p1: (u16, u16), p2: (u16, u16)) -> u8 {
    let angle = get_angle(p1, p2);
    let direction = round(angle / 45.0) as u8;
    if direction == 8 {
        0
    } else {
        direction
    }
}

/// Check if a Point (px, py) lies inside a circle (x, y, r)
pub fn in_circle((center_x, center_y, r): (u16, u16, u16), (px, py): (u16, u16)) -> bool {
    if r == 0 {
        return false;
    }
    let (center_x, center_y, r) = (center_x as f32, center_y as f32, r as f32);
    let (px, py) = (px as f32, py as f32);
    let dist_points = pow(px - center_x, 2) + pow(py - center_y, 2);
    let r2 = pow(r, 2);
    dist_points < r2
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rounds a float to the nearest decimal place.
    /// and then compares it to the expected value.
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr $(,)?) => {
            assert_eq!($a.round(), $b);
        };
    }

    #[test]
    fn in_range_works() {
        let p1 = (0, 0);
        let p2 = (0, 0);
        assert!(in_range(p1, p2, 0));

        for i in 0..SCREEN_DISTANCE {
            let p1 = (0, 0);
            let p2 = (i, i);
            assert!(in_range(p1, p2, SCREEN_DISTANCE));
        }

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE + 1, SCREEN_DISTANCE + 1);
        assert!(!in_range(p1, p2, SCREEN_DISTANCE));
    }

    #[test]
    fn in_screen_works() {
        let p1 = (0, 0);
        let p2 = (0, 0);
        assert!(in_screen(p1, p2));

        for i in 0..SCREEN_DISTANCE {
            let p1 = (0, 0);
            let p2 = (i, i);
            assert!(in_screen(p1, p2));
        }

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE + 1, SCREEN_DISTANCE + 1);
        assert!(!in_screen(p1, p2));
    }

    #[test]
    fn delta_works() {
        let p1 = (0, 0);
        let p2 = (0, 0);
        assert_eq!(delta(p1, p2), (0, 0));

        let p1 = (0, 0);
        let p2 = (1, 1);
        assert_eq!(delta(p1, p2), (1, 1));

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        assert_eq!(delta(p1, p2), (SCREEN_DISTANCE, SCREEN_DISTANCE));
    }

    #[test]
    fn within_elevation_works() {
        let new = 0;
        let initial = 0;
        assert!(within_elevation(new, initial));

        let new = 0;
        let initial = MAX_DIFFERENCE_IN_ELEVATION;
        assert!(within_elevation(new, initial));

        let new = MAX_DIFFERENCE_IN_ELEVATION;
        let initial = 0;
        assert!(!within_elevation(new, initial));

        let new = MAX_DIFFERENCE_IN_ELEVATION;
        let initial = MAX_DIFFERENCE_IN_ELEVATION;
        assert!(within_elevation(new, initial));
    }

    #[test]
    fn get_angle_works() {
        let p1 = (0, 0);
        let p2 = (0, 0);
        assert_approx_eq!(get_angle(p1, p2), 90.0);

        let p1 = (0, 0);
        let p2 = (1, 1);
        assert_approx_eq!(get_angle(p1, p2), 135.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        assert_approx_eq!(get_angle(p1, p2), 135.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, 0);
        assert_approx_eq!(get_angle(p1, p2), 90.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE * 2);
        assert_approx_eq!(get_angle(p1, p2), 153.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE);
        assert_approx_eq!(get_angle(p1, p2), 117.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE * 2);
        assert_approx_eq!(get_angle(p1, p2), 135.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, 0);
        assert_approx_eq!(get_angle(p1, p2), 90.0);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE * 2);
        assert_approx_eq!(get_angle(p1, p2), 135.0);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE);
        assert_approx_eq!(get_angle(p1, p2), 90.0);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (0, 0);
        assert_approx_eq!(get_angle(p1, p2), 135.0);
    }

    #[test]
    fn get_distance_works() {
        let p1 = (0, 0);
        let p2 = (0, 0);
        assert_approx_eq!(get_distance(p1, p2), 0.0);

        let p1 = (0, 0);
        let p2 = (1, 1);
        assert_approx_eq!(get_distance(p1, p2), 1.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        assert_approx_eq!(get_distance(p1, p2), 25.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, 0);
        assert_approx_eq!(get_distance(p1, p2), 18.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE * 2);
        assert_approx_eq!(get_distance(p1, p2), 40.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE);
        assert_approx_eq!(get_distance(p1, p2), 40.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE * 2);
        assert_approx_eq!(get_distance(p1, p2), 51.0);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, 0);
        assert_approx_eq!(get_distance(p1, p2), 36.0);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE * 2);
        assert_approx_eq!(get_distance(p1, p2), 25.0);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE);
        assert_approx_eq!(get_distance(p1, p2), 18.0);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (0, 0);
        assert_approx_eq!(get_distance(p1, p2), 25.0);
    }

    #[test]
    fn get_direction_sector_works() {
        let p1 = (0, 0);
        let p2 = (0, 0);
        assert_eq!(get_direction_sector(p1, p2), 2);

        let p1 = (0, 0);
        let p2 = (1, 1);
        assert_eq!(get_direction_sector(p1, p2), 3);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        assert_eq!(get_direction_sector(p1, p2), 3);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, 0);
        assert_eq!(get_direction_sector(p1, p2), 2);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE, SCREEN_DISTANCE * 2);
        assert_eq!(get_direction_sector(p1, p2), 3);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE);
        assert_eq!(get_direction_sector(p1, p2), 3);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE * 2);
        assert_eq!(get_direction_sector(p1, p2), 3);

        let p1 = (0, 0);
        let p2 = (SCREEN_DISTANCE * 2, 0);
        assert_eq!(get_direction_sector(p1, p2), 2);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE * 2);
        assert_eq!(get_direction_sector(p1, p2), 3);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (SCREEN_DISTANCE * 2, SCREEN_DISTANCE);
        assert_eq!(get_direction_sector(p1, p2), 2);

        let p1 = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        let p2 = (0, 0);
        assert_eq!(get_direction_sector(p1, p2), 3);
    }

    #[test]
    fn in_circle_works() {
        let center = (0, 0, 0);
        let point = (0, 0);
        assert!(!in_circle(center, point));

        let center = (0, 0, 0);
        let point = (1, 1);
        assert!(!in_circle(center, point));

        let center = (0, 0, 0);
        let point = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        assert!(!in_circle(center, point));

        let center = (0, 0, 0);
        let point = (SCREEN_DISTANCE + 1, SCREEN_DISTANCE + 1);
        assert!(!in_circle(center, point));

        let center = (0, 0, SCREEN_DISTANCE);
        let point = (0, 0);
        assert!(in_circle(center, point));

        let center = (0, 0, SCREEN_DISTANCE);
        let point = (1, 1);
        assert!(in_circle(center, point));

        let center = (0, 0, SCREEN_DISTANCE);
        let point = (SCREEN_DISTANCE, SCREEN_DISTANCE);
        assert!(!in_circle(center, point));

        let center = (0, 0, SCREEN_DISTANCE);
        let point = (SCREEN_DISTANCE + 1, SCREEN_DISTANCE + 1);
        assert!(!in_circle(center, point));

        let center = (0, 0, SCREEN_DISTANCE);
        let point = (4, 6);
        assert!(in_circle(center, point));
    }
}
