use crate::yabai::transport::Frame;

use super::Direction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn from_frame_center(frame: &Frame) -> Self {
        Self {
            x: frame.x + frame.width / 2.0,
            y: frame.y + frame.height / 2.0,
        }
    }

    pub fn dot(v1: Self, v2: Self) -> f32 {
        v1.x * v2.x + v1.y * v2.y
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl From<Direction> for Vector2D {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => Self { x: 0.0, y: -1.0 },
            Direction::East => Self { x: 1.0, y: 0.0 },
            Direction::South => Self { x: 0.0, y: 1.0 },
            Direction::West => Self { x: -1.0, y: 0.0 },
        }
    }
}

impl std::ops::Sub for Vector2D {
    type Output = Vector2D;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for Vector2D {
    type Output = Vector2D;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub fn find_closest_in_direction<T, Iter, F>(
    center: Vector2D,
    candidates: Iter,
    candidate_to_vector: F,
    direction: Vector2D,
) -> Option<T>
where
    F: Fn(&T) -> Vector2D,
    Iter: Iterator<Item = T>,
{
    let mut candidates_with_points: Vec<_> = candidates
        .filter_map(|candidate| {
            let candidate_direction: Vector2D = candidate_to_vector(&candidate) - center;

            let cos = Vector2D::dot(direction, candidate_direction)
                / (direction.length() * candidate_direction.length());

            // Make sure the candidate is noticeably in the right direction
            if cos < f32::EPSILON {
                None
            } else {
                let score = candidate_direction.length() / cos;
                Some((candidate, score))
            }
        })
        .collect();

    candidates_with_points.sort_by(|(_, score_a), (_, score_b)| {
        PartialOrd::partial_cmp(score_a, score_b).unwrap_or(std::cmp::Ordering::Less)
    });

    candidates_with_points
        .into_iter()
        .map(|(candidate, _)| candidate)
        .next()
}

pub fn get_candidates_in_direction<'a, 'b, T, Iter>(
    frame: &'a Frame,
    candidates: Iter,
    direction: Direction,
) -> Vec<&'b T>
where
    T: AsRef<Frame>,
    Iter: Iterator<Item = &'b T>,
{
    let check_frame_direction = match direction {
        Direction::West => Frame::is_west_of,
        Direction::East => Frame::is_east_of,
        Direction::North => Frame::is_north_of,
        Direction::South => Frame::is_south_of,
    };
    let check_overlap = match direction {
        Direction::West | Direction::East => Frame::overlaps_vertically,
        Direction::North | Direction::South => Frame::overlaps_horizontally,
    };

    candidates
        .filter(|&candidate| {
            let other_frame: &Frame = candidate.as_ref();

            check_frame_direction(other_frame, frame) && check_overlap(other_frame, frame)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_candidates_in_direction() {
        let frames = vec![
            Frame {
                x: 100.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            Frame {
                x: 200.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            Frame {
                x: 0.0,
                y: 100.0,
                width: 100.0,
                height: 100.0,
            },
            Frame {
                x: 100.0,
                y: -100.0,
                width: 100.0,
                height: 100.0,
            },
        ];

        let main_frame = Frame {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };

        let candidates = get_candidates_in_direction(&main_frame, frames.iter(), Direction::East);
        assert_eq!(candidates[0], &frames[0]);
        assert_eq!(candidates[1], &frames[1]);
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn find_closest() {
        let closest_vector = Vector2D { x: 1.0, y: 0.0 };

        let candidates = vec![
            Vector2D { x: 5.0, y: 2.0 },
            Vector2D { x: 17.2, y: 1.0 },
            closest_vector.clone(),
            Vector2D { x: 0.1, y: 10.0 },
            Vector2D { x: -0.1, y: -1.0 },
        ];

        let center = Vector2D { x: 0.0, y: 0.0 };

        let found_closest_vector = find_closest_in_direction(
            center,
            candidates.iter(),
            |&&vector| vector,
            Direction::East.into(),
        );

        assert_eq!(Some(&closest_vector), found_closest_vector);
    }
}
