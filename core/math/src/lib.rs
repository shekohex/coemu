/// This crate contains generic and handler specific calculation functions used
/// during packet and server processing. It contains calculations for screen
/// updating, attacking, exploit checking, etc.

pub const SCREEN_DISTANCE: u16 = 18;
pub const RADIAN_TO_DEGREE: f64 = 57.29;
pub const MAX_DIFFERENCE_IN_ELEVATION: u16 = 210;

/// This function returns true if an object is within the bounds of another
/// object's screen.
pub fn in_screen(p1: (u16, u16), p2: (u16, u16)) -> bool {
    let x1 = p1.0 as i16;
    let x2 = p2.0 as i16;
    let y1 = p1.1 as i16;
    let y2 = p2.1 as i16;
    let delta_x = (x2 - x1).abs();
    let delta_y = (y2 - y1).abs();
    let screen_distance = SCREEN_DISTANCE as i16;
    delta_x <= screen_distance && delta_y <= screen_distance
}

/// This function checks the elevation difference of two tiles.
pub fn within_elevation(new: u16, initial: u16) -> bool {
    new - initial <= MAX_DIFFERENCE_IN_ELEVATION
}

/// This function returns the angle for a jump or attack.
pub fn get_angle(p1: (u16, u16), p2: (u16, u16)) -> f64 {
    let x1 = p1.0 as i16;
    let x2 = p2.0 as i16;
    let y1 = p1.1 as i16;
    let y2 = p2.1 as i16;
    let delta_x = (x2 - x1).abs() as f64;
    let delta_y = (y2 - y1).abs() as f64;
    let angle = (delta_y.atan2(delta_x) * RADIAN_TO_DEGREE) + 90.0;
    if angle.is_sign_negative() {
        270.0 + (90.0 - angle.abs())
    } else {
        angle
    }
}

/// This function returns the distance between two objects.
pub fn get_distance(p1: (u16, u16), p2: (u16, u16)) -> f64 {
    let x1 = p1.0 as f64;
    let x2 = p2.0 as f64;
    let y1 = p1.1 as f64;
    let y2 = p2.1 as f64;
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

/// This function returns the direction for a jump or attack.
pub fn get_direction_sector(p1: (u16, u16), p2: (u16, u16)) -> u8 {
    let angle = get_angle(p1, p2);
    let direction = (angle / 45.0).round() as u8;
    if direction == 8 {
        0
    } else {
        direction
    }
}
