#![doc = include_str!("../README.md")]

use geo::{Coord, Geometry, MapCoords, Point, coord};
use geodesy::Direction::{Fwd, Inv};
use geodesy::Error as GeodesyError;
use geodesy::coord::{Coor2D, CoordinateTuple};
use geodesy::ctx::{Context, Minimal, OpHandle};

/// Catch-all for all definition or transformation errors.
#[derive(Debug)]
pub struct OMercError;

impl From<GeodesyError> for OMercError {
    fn from(_: GeodesyError) -> Self {
        OMercError
    }
}

/// An utility to convert `geo::Geometry` types back and forward between
/// geodesic degrees and cartesian meters.
pub struct OMercTransformer {
    context: Minimal,
    operation: OpHandle,
}

impl OMercTransformer {
    /// Creates a new transformer with a geodesic `anchor_point` as the new center
    /// point (0, 0) of a cartesian plane in meters, projected via Oblique Mercator.
    pub fn new(anchor_point: &Point) -> Result<Self, OMercError> {
        let mut context = Minimal::new();
        let operation = context.op(&format!(
            "omerc ellps=WGS84 latc={} lonc={} k_0=1.0 alpha=1e-06",
            anchor_point.y(),
            anchor_point.x()
        ))?;
        Ok(Self { context, operation })
    }

    /// Returns a new `geo::Geometry` with its coordinates transformed
    /// from geodesic (in degrees) to cartesian (in meters).
    pub fn to_cartesian(&self, geometry: &Geometry) -> Result<Geometry, OMercError> {
        Ok(
            geometry.try_map_coords(|c: Coord<f64>| -> Result<_, GeodesyError> {
                let mut data = [Coor2D::geo(c.y, c.x)];
                self.context.apply(self.operation, Fwd, &mut data)?;
                Ok(coord! {x: data[0].x(), y: data[0].y()})
            })?,
        )
    }

    /// Returns a new `geo::Geometry` with its coordinates transformed
    /// from  cartesian (in meters) to geodesic (in degrees).
    pub fn to_geodesic(&self, geometry: &Geometry) -> Result<Geometry, OMercError> {
        Ok(
            geometry.try_map_coords(|c: Coord<f64>| -> Result<_, GeodesyError> {
                let mut data = [Coor2D::raw(c.x, c.y)];
                self.context.apply(self.operation, Inv, &mut data)?;
                let xy = data[0].xy_to_degrees();
                Ok(coord! {x: xy.0, y: xy.1})
            })?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use geo::{Area, Centroid, Distance, Euclidean, polygon};

    #[test]
    fn roundtrip() {
        let praca_se = Point::new(-46.632913, -23.550617);
        let masp = Point::new(-46.655945, -23.561442);
        let transformer = OMercTransformer::new(&praca_se).unwrap();
        let c_masp = transformer.to_cartesian(&masp.into()).unwrap();
        let g_masp: Point = transformer
            .to_geodesic(&c_masp.into())
            .unwrap()
            .try_into()
            .unwrap();
        assert_approx_eq!(g_masp.x(), masp.x());
        assert_approx_eq!(g_masp.y(), masp.y());
    }

    #[test]
    fn distance_in_meters() {
        let center_of_berlin = Point::new(13.409408, 52.520842);
        let brandenburg_gate = Point::new(13.377701, 52.516262);
        let transformer = OMercTransformer::new(&center_of_berlin).unwrap();
        let c_center_of_berlin = transformer.to_cartesian(&center_of_berlin.into()).unwrap();
        let c_brandenburg_gate = transformer.to_cartesian(&brandenburg_gate.into()).unwrap();
        let distance = Euclidean.distance(&c_center_of_berlin, &c_brandenburg_gate);
        assert_approx_eq!(distance, 2211.840487); // meters.
    }

    #[test]
    fn area_of_polygon() {
        let museum_island = polygon![
            (x: 13.393639, y: 52.522081),
            (x: 13.397824, y: 52.519285),
            (x: 13.403055, y: 52.512100),
            (x: 13.405072, y: 52.511486),
            (x: 13.411649, y: 52.513873),
            (x: 13.406418, y: 52.514692),
            (x: 13.401411, y: 52.519580),
            (x: 13.398160, y: 52.521513),
            (x: 13.393639, y: 52.522081),
        ];
        let transformer = OMercTransformer::new(&museum_island.centroid().unwrap()).unwrap();
        let c_museum_island = transformer.to_cartesian(&museum_island.into()).unwrap();
        assert_approx_eq!(c_museum_island.unsigned_area(), 377902.962989); // square meters.
    }
}
