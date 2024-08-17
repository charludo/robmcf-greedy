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
            let path = shortest_path(&prev, s, t);
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
