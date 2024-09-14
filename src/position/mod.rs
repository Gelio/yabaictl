mod direction;
mod vector_2d;

pub use direction::Direction;
use log::trace;

use self::vector_2d::{find_closest_in_direction, get_candidates_in_direction, Vector2D};
use crate::yabai::transport::Frame;

pub fn get_element_to_focus<'a, T: std::fmt::Debug + AsRef<Frame>>(
    focused_frame: &Frame,
    elements: &'a [T],
    direction: Direction,
) -> Option<&'a T> {
    trace!("Looking for candidates to focus in direction {direction:?} from {focused_frame:?}");
    let candidates_in_direction =
        get_candidates_in_direction(focused_frame, elements.iter(), direction);
    trace!("Found candidates: {candidates_in_direction:#?}");

    find_closest_in_direction(
        Vector2D::from_frame_center(focused_frame),
        candidates_in_direction.into_iter(),
        |window| Vector2D::from_frame_center(window.as_ref()),
        direction.into(),
    )
}
