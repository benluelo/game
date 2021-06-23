use crate::bounded_int::BoundedInt;

use super::Point;

pub(crate) trait PointIndex<T> {
    type Output;
    fn at<const L: i32, const H: i32>(
        &self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &Self::Output;
    fn at_mut<const L: i32, const H: i32>(
        &mut self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &mut Self::Output;
}

impl<T /* , const L: i32, const H: i32 */> PointIndex<T> for Vec<T> {
    type Output = T;

    fn at<const L: i32, const H: i32>(
        &self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &Self::Output {
        &self[((*point.row.get() * *width) + *point.column.get()) as usize]
    }

    fn at_mut<const L: i32, const H: i32>(
        &mut self,
        point: Point,
        width: BoundedInt<{ L }, { H }>,
    ) -> &mut Self::Output {
        &mut self[((*point.row.get() * *width) + *point.column.get()) as usize]
    }
}

#[cfg(test)]
mod test_point_index {
    use std::convert::TryInto;

    use crate::dungeon::{
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
