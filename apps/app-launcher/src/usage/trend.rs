//! Shape-preserving monitor curves through the actual sampled values.
//!
//! Steffen tangents keep plateaus and local extrema while sharing one derivative
//! at each knot. Unlike a moving average, this never changes a displayed sample.
//! Geometry is calculated in f64 before handing pixel coordinates to tiny_gfx.
use tiny_flutter::tiny_gfx::PathBuilder;

type Point = (f32, f32);

// Inputs are already chart pixel coordinates. Reject implausible coordinates
// before tiny_gfx's f32 midpoint/distance calculations can overflow.
const MAX_COORDINATE: f32 = 1_000_000.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Segment {
    pub(crate) start: Point,
    pub(crate) first: Point,
    pub(crate) second: Point,
    pub(crate) end: Point,
}

/// Interpolate finite, increasing-x runs without bridging invalid samples.
///
/// Empty input produces an empty builder; an isolated sample is a MoveTo. NaN,
/// infinity, implausibly large coordinates, duplicate x and decreasing x end
/// the current run. No sorting, averaging or synthetic values are introduced.
pub fn curve(points: &[Point]) -> PathBuilder {
    let mut path = PathBuilder::new();
    for run in valid_runs(points) {
        if run.len() == 1 {
            path.move_to(run[0].0, run[0].1);
        }
        for (index, segment) in segments(run).into_iter().enumerate() {
            if index == 0 {
                path.move_to(segment.start.0, segment.start.1);
            }
            path.cubic_to(
                segment.first.0,
                segment.first.1,
                segment.second.0,
                segment.second.1,
                segment.end.0,
                segment.end.1,
            );
        }
    }
    path
}

fn valid(point: Point) -> bool {
    point.0.is_finite()
        && point.1.is_finite()
        && point.0.abs() <= MAX_COORDINATE
        && point.1.abs() <= MAX_COORDINATE
}

fn valid_runs(points: &[Point]) -> impl Iterator<Item = &[Point]> {
    let mut index = 0;
    std::iter::from_fn(move || {
        while index < points.len() && !valid(points[index]) {
            index += 1;
        }
        let start = index;
        if start == points.len() {
            return None;
        }
        index += 1;
        while index < points.len() && valid(points[index]) && points[index].0 > points[index - 1].0
        {
            index += 1;
        }
        Some(&points[start..index])
    })
}

/// Kept crate-private so the geometry can be verified independently of raster AA.
pub(crate) fn segments(points: &[Point]) -> Vec<Segment> {
    if points.len() < 2 {
        return Vec::new();
    }
    let intervals = points
        .windows(2)
        .map(|p| f64::from(p[1].0) - f64::from(p[0].0))
        .collect::<Vec<_>>();
    let secants = points
        .windows(2)
        .zip(&intervals)
        .map(|(p, h)| (f64::from(p[1].1) - f64::from(p[0].1)) / h)
        .collect::<Vec<_>>();
    let mut tangents = vec![0.0; points.len()];
    for i in 1..points.len() - 1 {
        let a = secants[i - 1];
        let b = secants[i];
        if (a > 0.0 && b > 0.0) || (a < 0.0 && b < 0.0) {
            let h0 = intervals[i - 1];
            let h1 = intervals[i];
            let average = (a * h1 + b * h0) / (h0 + h1);
            tangents[i] = a.signum() * (2.0 * a.abs().min(b.abs())).min(average.abs());
        }
    }
    if points.len() == 2 {
        tangents.fill(secants[0]);
    } else {
        // Half of the three-point derivative estimate at each endpoint. The
        // neighbouring Steffen tangent bounds this to [0.5, 1.5] * secant.
        tangents[0] = (3.0 * secants[0] - tangents[1]) * 0.5;
        let last = points.len() - 1;
        tangents[last] = (3.0 * secants[last - 1] - tangents[last - 1]) * 0.5;
    }
    points
        .windows(2)
        .enumerate()
        .map(|(i, pair)| {
            let dx = intervals[i] / 3.0;
            let x0 = f64::from(pair[0].0);
            let y0 = f64::from(pair[0].1);
            let x1 = f64::from(pair[1].0);
            let y1 = f64::from(pair[1].1);
            let lo = y0.min(y1);
            let hi = y0.max(y1);
            Segment {
                start: pair[0],
                first: (
                    (x0 + dx) as f32,
                    (y0 + tangents[i] * dx).clamp(lo, hi) as f32,
                ),
                second: (
                    (x1 - dx) as f32,
                    (y1 - tangents[i + 1] * dx).clamp(lo, hi) as f32,
                ),
                end: pair[1],
            }
        })
        .collect()
}
