use crate::matrix::Matrix;

pub(crate) fn floyd_warshall(
    capacities: &Matrix<usize>,
    costs: &Matrix<usize>,
) -> (Matrix<usize>, Matrix<Option<usize>>) {
    let mut dist: Matrix<usize> =
        Matrix::filled_with(usize::MAX, costs.num_rows(), costs.num_columns());
    let mut prev: Matrix<Option<usize>> =
        Matrix::filled_with(None, costs.num_rows(), costs.num_columns());

    for (x, y) in capacities
        .indices()
        .filter(|(x, y)| *capacities.get(*x, *y) > 0)
    {
        log::trace!("dist {} -> {} is {}", x, y, costs.get(x, y));
        dist.set(x, y, *costs.get(x, y));
        prev.set(x, y, Some(x));
    }
    for v in 0..capacities.num_rows() {
        log::trace!("pred. of {} is {} with distance 0", v, v);
        dist.set(v, v, 0);
        prev.set(v, v, Some(v));
    }
    for k in 0..capacities.num_rows() {
        for i in 0..capacities.num_rows() {
            for j in 0..capacities.num_rows() {
                if *dist.get(i, k) < usize::MAX
                    && *dist.get(k, j) < usize::MAX
                    && *dist.get(i, j) > dist.get(i, k) + dist.get(k, j)
                {
                    log::trace!(
                        "new dist {} -> {} is {}",
                        i,
                        j,
                        dist.get(i, k) + dist.get(k, j)
                    );
                    dist.set(i, j, dist.get(i, k) + dist.get(k, j));
                    prev.set(i, j, *prev.get(k, j));
                }
            }
        }
    }

    log::debug!(
        "Floyd-Warshall resulted in distance map\n{}\nand predecessor map\n{}",
        dist,
        prev
    );

    (dist, prev)
}

pub(crate) fn invert_predecessors(prev: &Matrix<Option<usize>>) -> Matrix<usize> {
    let mut succ: Matrix<usize> =
        Matrix::filled_with(usize::MAX, prev.num_rows(), prev.num_columns());

    prev.indices().for_each(|(s, t)| {
        if *succ.get(s, t) == usize::MAX {
            let path = shortest_path(prev, s, t);
            for i in 0..path.len() {
                succ.set(path[i], t, if i + 1 < path.len() { path[i + 1] } else { t });
                if *succ.get(path[i], t) != usize::MAX {
                    break;
                }
            }
        }
    });

    log::debug!(
        "Predecessor map has been inverted into the following succcessor map:\n{}",
        succ
    );

    succ
}

fn shortest_path(prev: &Matrix<Option<usize>>, s: usize, mut t: usize) -> Vec<usize> {
    let mut p = match prev.get(s, t) {
        Some(_) => vec![t],
        None => return vec![],
    };

    while s != t {
        t = prev.get(s, t).expect("");
        p.push(t);
    }

    p.reverse();
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (
        Matrix<usize>,
        Matrix<usize>,
        Matrix<usize>,
        Matrix<Option<usize>>,
        Matrix<usize>,
    ) {
        let capacities: Matrix<usize> =
            Matrix::from_elements(&vec![0, 0, 2, 1, 0, 2, 3, 2, 0], 3, 3);
        let costs: Matrix<usize> = Matrix::from_elements(&vec![0, 0, 3, 4, 0, 6, 7, 8, 0], 3, 3);
        let distance_map: Matrix<usize> =
            Matrix::from_elements(&vec![0, 11, 3, 4, 0, 6, 7, 8, 0], 3, 3);
        let predecessor_map: Matrix<Option<usize>> = Matrix::from_elements(
            &vec![
                Some(0),
                Some(2),
                Some(0),
                Some(1),
                Some(1),
                Some(1),
                Some(2),
                Some(2),
                Some(2),
            ],
            3,
            3,
        );
        let successor_map: Matrix<usize> =
            Matrix::from_elements(&vec![0, 2, 2, 0, 1, 2, 0, 1, 2], 3, 3);

        (
            capacities,
            costs,
            distance_map,
            predecessor_map,
            successor_map,
        )
    }

    #[test]
    fn test_floyd_warshall_distances() {
        let (capacities, costs, distance_map, _, _) = setup();
        let (dist, _) = floyd_warshall(&capacities, &costs);

        assert_eq!(distance_map, dist);
    }

    #[test]
    fn test_floyd_warshall_predecessors() {
        let (capacities, costs, _, predecessor_map, _) = setup();
        let (_, prev) = floyd_warshall(&capacities, &costs);

        assert_eq!(predecessor_map, prev);
    }

    #[test]
    fn test_shortest_path_0_0() {
        let (_, _, _, predecessor_map, _) = setup();
        let path = shortest_path(&predecessor_map, 0, 0);

        assert_eq!(vec![0], path);
    }

    #[test]
    fn test_shortest_path_0_1() {
        let (_, _, _, predecessor_map, _) = setup();
        let path = shortest_path(&predecessor_map, 0, 1);

        assert_eq!(vec![0, 2, 1], path);
    }

    #[test]
    fn test_invert_predecessors() {
        let (_, _, _, predecessor_map, successor_map) = setup();
        let succ = invert_predecessors(&predecessor_map);

        assert_eq!(successor_map, succ);
    }
}
