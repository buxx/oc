use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Xy(pub u64, pub u64);

impl From<Xy> for (u64, u64) {
    fn from(value: Xy) -> Self {
        (value.0, value.1)
    }
}
impl From<(isize, isize)> for Xy {
    fn from((x, y): (isize, isize)) -> Self {
        Self(x as u64, y as u64)
    }
}

impl Xy {
    pub fn distance(self, other: Self) -> f32 {
        ((other.0 as f32 - self.0 as f32).powi(2) + (other.1 as f32 - self.1 as f32).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_line_x() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(1, 0);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_line_x_neg() {
        // Given
        let a = Xy(1, 0);
        let b = Xy(0, 0);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_line_y() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(0, 1);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_distance_diag() {
        // Given
        let a = Xy(0, 0);
        let b = Xy(1, 1);

        // When
        let distance = a.distance(b);

        // Then
        assert_eq!(distance, 1.4142135);
    }
}
