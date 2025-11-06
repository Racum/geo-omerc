# Oblique Mercator Transformer for geo::Geometry Types

Convert geo types between geodesic degrees and cartesian meters using Oblique Mercator.

This uses the [Oblique Mercator](https://en.wikipedia.org/wiki/Oblique_Mercator_projection)
projection to define arbitrary zero points (0, 0) on the globe, making
deformation near that point negligible; and at the same time converting
the units to meters, that allows for precise measurements of distances and areas.

Under the hood this uses the [geodesy](https://crates.io/crates/geodesy) crate for the projection calculation of each coordinate, but with an ergonomic wrapper to handle full [geo::Geometry](https://docs.rs/geo/latest/geo/geometry/enum.Geometry.html) types (points, lines, polygons, etc).

## Usage

Installation:

```shell
$ cargo add geo-omerc
```

**OMercTransformer** methods:

- `new()`: creates a new transformer with a geodesic anchor point defined as the new cartesian center.
- `to_cartesian()`: converts geometries with geodesic coordinates (in degrees) into ones with cartesian coordinates (in meters).
- `to_geodesic()`: converts geometries with cartesian coordinates (in meters) into ones with geodesic coordinates (in degrees).

This uses a struct with methods instead of just functions to cache the operation definition, that, once created, could be reused to transform many geometries. The two transformation methods handle `geo::Geometry` as both input and output (wrapped in a Result).

## Examples

```rust
use geo::{Point, polygon, Distance, Euclidean, Area};
use geo_omerc::OMercTransformer;

let coliseum = Point::new(12.49233, 41.89018);
let pantheon = Point::new(12.47683, 41.89856);

// Defines a transformer with the Coliseum as the center (0, 0):
let transformer = OMercTransformer::new(&coliseum).unwrap();


// Roundtrip: --------------------------------------------------------

let pantheon_m = transformer.to_cartesian(&pantheon.into()).unwrap();
println!("{:?}", pantheon_m);
// POINT(-1286.2243529009943 930.893116932613) -> in meters.

let pantheon_d = transformer.to_geodesic(&pantheon_m.into()).unwrap();
println!("{:?}", pantheon_d);
// POINT(12.476829999999996 41.8985600000268) -> in degrees.


// Distance: ---------------------------------------------------------

let coliseum_m = transformer.to_cartesian(&coliseum.into()).unwrap();
let pantheon_m = transformer.to_cartesian(&pantheon.into()).unwrap();
let distance = Euclidean.distance(&coliseum_m, &pantheon_m);
println!("{}", distance);
// 1587.7452822004639 -> in meters.


// Area: -------------------------------------------------------------

let circus_maximus = polygon![
    (x: 12.4833, y: 41.8881),
    (x: 12.4823, y: 41.8872),
    (x: 12.4879, y: 41.8838),
    (x: 12.4891, y: 41.8849),
    (x: 12.4833, y: 41.8881),
];
let circus_maximus_m = transformer.to_cartesian(&circus_maximus.into()).unwrap();
println!("{}", circus_maximus_m.unsigned_area());
// 86010.79776633419 -> in square meters.
```

