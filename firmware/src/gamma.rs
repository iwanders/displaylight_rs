use crate::types::RGB;

/// Gamma tables to map ws2811 rgb leds properly.
pub struct Gamma {
    gamma_r: [u8; 256],
    gamma_g: [u8; 256],
    gamma_b: [u8; 256],
}
impl Default for Gamma {
    fn default() -> Gamma {
        Gamma::linear()
    }
}

const fn create_linear() -> [u8; 256] {
    let mut lookup = [0; 256];
    let mut i = 0usize;
    while i < 256 {
        lookup[i] = i as u8;
        i += 1;
    }
    lookup
}

// Looks like nostd doesn't have powf.
/*
fn create_exponential(exponent: f32) ->  [u8; 256] {
    let mut lookup = [0; 256];
    let mut i = 0usize;
    while i < 256 {
        let v = ((i as f32) / 255.0).powf(exponent) * 256.0 + 0.5;
        lookup[i] = v as u8;
        i += 1;
    }
    lookup
}
*/

const fn create_exponential_gamma_1_0() -> [u8; 256] {
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
        71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93,
        94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112,
        113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130,
        131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148,
        149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166,
        167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184,
        185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202,
        203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220,
        221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238,
        239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
    ]
}

const fn create_exponential_gamma_1_3() -> [u8; 256] {
    [
        0, 0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 8, 8, 9, 9, 10, 11, 11, 12, 12, 13, 14,
        14, 15, 16, 16, 17, 18, 19, 19, 20, 21, 21, 22, 23, 24, 24, 25, 26, 27, 28, 28, 29, 30, 31,
        31, 32, 33, 34, 35, 36, 36, 37, 38, 39, 40, 41, 41, 42, 43, 44, 45, 46, 47, 47, 48, 49, 50,
        51, 52, 53, 54, 55, 56, 57, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72,
        73, 74, 75, 76, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 88, 89, 90, 91, 92, 93, 94, 95,
        96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 112, 113, 114, 115,
        116, 117, 118, 119, 120, 121, 122, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 135,
        136, 137, 138, 139, 140, 141, 143, 144, 145, 146, 147, 148, 149, 151, 152, 153, 154, 155,
        156, 157, 159, 160, 161, 162, 163, 164, 166, 167, 168, 169, 170, 172, 173, 174, 175, 176,
        178, 179, 180, 181, 182, 184, 185, 186, 187, 188, 190, 191, 192, 193, 194, 196, 197, 198,
        199, 201, 202, 203, 204, 206, 207, 208, 209, 210, 212, 213, 214, 215, 217, 218, 219, 220,
        222, 223, 224, 226, 227, 228, 229, 231, 232, 233, 234, 236, 237, 238, 240, 241, 242, 243,
        245, 246, 247, 249, 250, 251, 252, 254, 255,
    ]
}

const fn create_exponential_gamma_1_6() -> [u8; 256] {
    [
        0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7, 7, 7, 8,
        8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 18, 18, 19, 19, 20,
        21, 21, 22, 23, 23, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 31, 32, 33, 34, 34, 35, 36,
        37, 38, 38, 39, 40, 41, 42, 42, 43, 44, 45, 46, 46, 47, 48, 49, 50, 51, 52, 53, 53, 54, 55,
        56, 57, 58, 59, 60, 61, 62, 63, 64, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77,
        78, 79, 80, 81, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 97, 98, 99, 100, 101,
        102, 103, 104, 106, 107, 108, 109, 110, 111, 113, 114, 115, 116, 117, 119, 120, 121, 122,
        123, 125, 126, 127, 128, 130, 131, 132, 133, 135, 136, 137, 138, 140, 141, 142, 143, 145,
        146, 147, 149, 150, 151, 153, 154, 155, 157, 158, 159, 161, 162, 163, 165, 166, 167, 169,
        170, 171, 173, 174, 176, 177, 178, 180, 181, 183, 184, 185, 187, 188, 190, 191, 193, 194,
        196, 197, 198, 200, 201, 203, 204, 206, 207, 209, 210, 212, 213, 215, 216, 218, 219, 221,
        222, 224, 225, 227, 228, 230, 231, 233, 235, 236, 238, 239, 241, 242, 244, 245, 247, 249,
        250, 252, 253, 255,
    ]
}

impl Gamma {
    pub fn linear() -> Self {
        Gamma {
            gamma_r: create_linear(),
            gamma_g: create_linear(),
            gamma_b: create_linear(),
        }
    }

    pub fn correction() -> Self {
        Gamma {
            gamma_r: create_exponential_gamma_1_0(),
            gamma_g: create_exponential_gamma_1_3(),
            gamma_b: create_exponential_gamma_1_6(),
        }
    }

    pub fn apply(&self, colors: &mut [RGB]) {
        for c in colors.iter_mut() {
            c.r = self.gamma_r[c.r as usize];
            c.g = self.gamma_g[c.g as usize];
            c.b = self.gamma_b[c.b as usize];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_checks() {
        let v = create_linear();
        for i in 0..256 {
            assert_eq!(v[i], i as u8);
        }
    }
}