//! Geometry checks do not depend on renderer tolerance or screen appearance.
#[path = "../src/usage/trend.rs"]
mod trend;

fn sample(s: trend::Segment, t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let c = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    let points = [s.start, s.first, s.second, s.end];
    points.iter().zip(c).fold((0.0, 0.0), |(x, y), (p, v)| {
        (x + f64::from(p.0) * v, y + f64::from(p.1) * v)
    })
}

fn assert_shape(points: &[(f32, f32)]) {
    let segments = trend::segments(points);
    assert_eq!(segments.len(), points.len().saturating_sub(1));
    for (i, s) in segments.iter().copied().enumerate() {
        assert_eq!(s.start, points[i]);
        assert_eq!(s.end, points[i + 1]);
        let lo = f64::from(s.start.1.min(s.end.1));
        let hi = f64::from(s.start.1.max(s.end.1));
        let mut previous = f64::from(s.start.1);
        for n in 0..=1000 {
            let (x, y) = sample(s, f64::from(n) / 1000.0);
            assert!(x.is_finite() && y.is_finite());
            assert!(x >= f64::from(s.start.0) - 1e-6);
            assert!(x <= f64::from(s.end.0) + 1e-6);
            assert!(
                y >= lo - 1e-6 && y <= hi + 1e-6,
                "{i}: {y} outside [{lo}, {hi}]"
            );
            if s.start.1 <= s.end.1 {
                assert!(y >= previous - 1e-6, "{i}: rising segment turned downward");
            } else {
                assert!(y <= previous + 1e-6, "{i}: falling segment turned upward");
            }
            previous = y;
        }
    }
}

#[test]
fn hourly_sharp_peaks_stay_within_real_samples() {
    let amounts = [0., 0., 120., 8., 300., 299., 301., 0., 0., 1., 230., 4.];
    let points = amounts
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f32 * 22., v))
        .collect::<Vec<_>>();
    assert_shape(&points);
    let poly = trend::curve(&points).finish().unwrap().flatten(0.01);
    for p in points {
        assert!(
            poly[0].iter().any(|q| (q.x, q.y) == p),
            "sample {p:?} missing"
        );
    }
}

#[test]
fn irregular_intervals_plateaus_and_extreme_ratios_do_not_overshoot() {
    for points in [
        vec![(0., 10.), (1., 10.), (40., 10.), (41., 700.), (900., 700.)],
        vec![(0., 0.), (1., 100_000.), (2., 100_001.), (1000., 0.)],
        vec![(0., 800.), (1., 1.), (2., 0.), (20., 1.), (21., 800.)],
        vec![(0., 0.), (0.001, 1.), (100., 2.), (100.002, 0.)],
    ] {
        assert_shape(&points);
    }
}

#[test]
fn shared_tangents_are_c1_continuous_even_with_irregular_spacing() {
    let points = [
        (0., 190.),
        (19., 152.),
        (77., 44.),
        (79., 90.),
        (151., 120.),
        (170., 120.),
    ];
    let segments = trend::segments(&points);
    for pair in segments.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let left = (a.end.1 - a.second.1) / (a.end.0 - a.second.0);
        let right = (b.first.1 - b.start.1) / (b.first.0 - b.start.0);
        assert!(
            (left - right).abs() <= 0.0001 * left.abs().max(right.abs()).max(1.),
            "{left} != {right}"
        );
    }
    assert_shape(&points);
}

#[test]
fn sparse_linear_and_flat_series_are_exact() {
    assert!(trend::curve(&[]).finish().is_none());
    let singleton = trend::curve(&[(12., 44.)]).finish().unwrap().flatten(0.1);
    assert_eq!(singleton.len(), 1);
    assert_eq!(singleton[0].len(), 1);
    assert_eq!((singleton[0][0].x, singleton[0][0].y), (12., 44.));
    for points in [
        vec![(0., 8.), (100., 208.)],
        vec![(0., 0.), (100., 0.), (200., 0.)],
    ] {
        assert_shape(&points);
        for s in trend::segments(&points) {
            for n in 0..=10 {
                let (x, y) = sample(s, f64::from(n) / 10.0);
                let expected = f64::from(s.start.1)
                    + (x - f64::from(s.start.0)) * f64::from(s.end.1 - s.start.1)
                        / f64::from(s.end.0 - s.start.0);
                assert!((y - expected).abs() < 0.00001);
            }
        }
    }
}

#[test]
fn invalid_coordinates_break_runs_without_bridging_or_nonfinite_geometry() {
    let points = [
        (0., 10.),
        (10., 20.),
        (20., f32::NAN),
        (30., 20.),
        (40., 10.),
        (f32::INFINITY, 0.),
        (50., 50.),
        (50., 60.),
        (49., 70.),
        (60., 80.),
        (f32::MAX, f32::MAX),
        (70., 5.),
        (80., 8.),
    ];
    let polylines = trend::curve(&points).finish().unwrap().flatten(0.01);
    let ends = polylines
        .iter()
        .map(|line| {
            for p in line {
                assert!(p.x.is_finite() && p.y.is_finite());
            }
            let a = line.first().unwrap();
            let b = line.last().unwrap();
            ((a.x, a.y), (b.x, b.y))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ends,
        vec![
            ((0., 10.), (10., 20.)),
            ((30., 20.), (40., 10.)),
            ((50., 50.), (50., 50.)),
            ((50., 60.), (50., 60.)),
            ((49., 70.), (60., 80.)),
            ((70., 5.), (80., 8.)),
        ]
    );
}

#[test]
fn local_extrema_and_plateaus_have_horizontal_shared_tangents() {
    let points = [(0., 0.), (20., 80.), (40., 80.), (60., 10.), (80., 60.)];
    let segments = trend::segments(&points);
    for knot in 1..points.len() - 1 {
        assert_eq!(segments[knot - 1].second.1, points[knot].1);
        assert_eq!(segments[knot].first.1, points[knot].1);
    }
    assert_shape(&points);
}
