use std::convert::TryInto;

use bounded_int::BoundedInt;

use crate::{
    floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE},
    Column, Row,
};

use super::Point;

/// A 1-dimensional type representing a 2-dimensional grid, indexable by a
/// [`Point`].
///
/// This trait is implemented for [`Vec<T>`] and [`std::slice`]s, but any
/// indexable type could be able to implement this as well.
///
/// This trait assumes that the underlying implementation uses **row-major**
/// order, and as such the width of the 2D grid must be provided.
///
/// The equation to find the 1D index in a 2D grid, provided the width of the
/// grid, is `(row * width) + column`.
///
/// For a grid with width = 4 and height = 3, one could think of it as being
/// represented internally like this:
/// ```txt
/// [a, b, c, d, e, f, g, h, i, j, k, l]
///  ^--------^  ^--------^  ^--------^
/// ```

/// Where that would translate to this 2D grid:
/// ```txt
/// [
///     [a, b, c, d],
///     [e, f, g, h],
///     [i, j, k, l],
/// ]
/// ```
///
/// To access the item at
/// ```txt
/// Point {
///     row: 1,
///     column: 2,
/// }
/// ```
/// the math would work like this:
/// ```txt
/// [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,11]
/// [a, b, c, d, e, f, g, h, i, j, k, l ]
///
/// index = (row * width) + column
/// index = (1 * 4) + 2
/// index = 6
/// ```
///
/// Looking at the 2D representation again, we can see that this makes sense:
/// ```txt
///      0  1  2  3
///   0 [a, b, c, d],
///   1 [e, f, g, h],
/// ┌----------^
/// | 2 [i, j, k, l],
/// |
/// └---> row 1, column 2
/// ```
///
/// # Examples
/// ```rust
/// use std::convert::TryInto;
///
/// use bounded_int::BoundedInt;
///
/// use dungeon::{
///     point_index::PointIndex,
///     Point, Column, Row,
/// };
///
/// const WIDTH: i32 = 4;
/// const HEIGHT: i32 = 5;
/// const MAP: [u8; (WIDTH * HEIGHT) as usize] = [
///     // <- width ->
///     0, 0, 0, 0, //   ^
///     0, 0, 0, 0, //   |
///     0, 0, 0, 0, // height
///     0, 0, 1, 0, //   |
///     0, 0, 0, 0, //   v
/// ];
///
/// let mut map = Vec::from(MAP);
///
/// assert!(matches!(
///     map.at(
///         Point {
///             row: Row::new(3.try_into().unwrap()),
///             column: Column::new(2.try_into().unwrap()),
///         },
///         BoundedInt::<0, { HEIGHT * WIDTH }>::new(WIDTH).unwrap()
///     ),
///     &1
/// ));
///
/// *map.at_mut(
///     Point {
///         row: Row::new(2.try_into().unwrap()),
///         column: Column::new(1.try_into().unwrap()),
///     },
///     BoundedInt::<0, { HEIGHT * WIDTH }>::new(WIDTH).unwrap(),
/// ) = 1;
///
/// assert!(matches!(
///     map.at(
///         Point {
///             row: Row::new(2.try_into().unwrap()),
///             column: Column::new(1.try_into().unwrap()),
///         },
///         BoundedInt::<0, { HEIGHT * WIDTH }>::new(WIDTH).unwrap(),
///     ),
///     &1
/// ));
/// ```
pub trait PointIndex {
    /// The type returned by this trait's methods.
    ///
    /// Will most often be the type that the implementating collection is
    /// generic over.
    type Output;

    /// Returns a reference to the item at that point.
    fn at<const L: i32, const H: i32>(
        &self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &Self::Output;

    /// Returns a mutable reference to the item at that point.
    fn at_mut<const L: i32, const H: i32>(
        &mut self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &mut Self::Output;
}

impl<T> PointIndex for Vec<T> {
    type Output = T;

    fn at<const L: i32, const H: i32>(
        &self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &Self::Output {
        &self[((point.row.get().as_unbounded() * width.as_unbounded())
            + point.column.get().as_unbounded()) as usize]
    }

    fn at_mut<const L: i32, const H: i32>(
        &mut self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &mut Self::Output {
        &mut self[((point.row.get().as_unbounded() * width.as_unbounded())
            + point.column.get().as_unbounded()) as usize]
    }
}

impl<T> PointIndex for [T] {
    type Output = T;

    fn at<const L: i32, const H: i32>(
        &self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &Self::Output {
        &self[((point.row.get().as_unbounded() * width.as_unbounded())
            + point.column.get().as_unbounded()) as usize]
    }

    fn at_mut<const L: i32, const H: i32>(
        &mut self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &mut Self::Output {
        &mut self[((point.row.get().as_unbounded() * width.as_unbounded())
            + point.column.get().as_unbounded()) as usize]
    }
}

// TODO: Where should this function go?
/// Returns an iterator over all of the [`Point`]s contained in a map of the specified `width` and `height`.
pub(crate) fn iter_points(
    width: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
    height: BoundedInt<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>,
) -> impl Iterator<Item = Point> {
    width
        .expand_lower::<0>()
        .range_from(0.try_into().unwrap())
        .flat_map(move |column| {
            height
                .expand_lower::<0>()
                .range_from(0.try_into().unwrap())
                .map(move |row| Point {
                    column: Column::new(column),
                    row: Row::new(row),
                })
        })
}

#[cfg(test)]
mod test_point_index {
    use std::convert::TryInto;

    use crate::{
        floor_builder::{MAX_FLOOR_SIZE, MIN_FLOOR_SIZE},
        Column, Row,
    };

    use super::*;

    #[test]
    fn test_point_index() {
        const WIDTH: i32 = 10;
        const HEIGHT: i32 = 20;
        #[rustfmt::skip]
        const MAP: [i32; (WIDTH * HEIGHT) as usize] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mut map = Vec::from(MAP);

        assert!(matches!(
            map.at(
                Point {
                    row: Row(3.try_into().unwrap()),
                    column: Column(4.try_into().unwrap()),
                },
                BoundedInt::<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>::new(WIDTH).unwrap()
            ),
            &1
        ));

        *map.at_mut(
            Point {
                row: Row(7.try_into().unwrap()),
                column: Column(2.try_into().unwrap()),
            },
            BoundedInt::<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>::new(WIDTH).unwrap(),
        ) = 1;

        assert!(matches!(
            map.at(
                Point {
                    row: Row(7.try_into().unwrap()),
                    column: Column(2.try_into().unwrap()),
                },
                BoundedInt::<MIN_FLOOR_SIZE, MAX_FLOOR_SIZE>::new(WIDTH).unwrap(),
            ),
            &1
        ));
    }
}
