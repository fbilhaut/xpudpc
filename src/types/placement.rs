/// Aircraft placement / start type for [`XPlaneClient::place_aircraft`].
///
/// Use [`StartType::SpecifyLatLonEle`] to position the aircraft at arbitrary
/// coordinates; set `airport_id` to `""` in that case.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartType {
    RepeatLast = 5,
    SpecifyLatLonEle = 6,
    GeneralArea = 7,
    NearestAirport = 8,
    SnapLoad = 9,
    Ramp = 10,
    Takeoff = 11,
    VfrApproach = 12,
    IfrApproach = 13,
    GrassStrip = 14,
    DirtStrip = 15,
    GravelStrip = 16,
    SeaplaneStart = 17,
    Helipad = 18,
    CarrierCatshot = 19,
    GliderTow = 20,
    GliderWinch = 21,
    Formation = 22,
    RefuelBoom = 23,
    RefuelBasket = 24,
    B52Drop = 25,
    PiggyBack = 26,
    CarrierApproach = 27,
    FrigateApproach = 28,
    SmallOilRig = 29,
    LargeOilPlatform = 30,
    ForestFire = 31,
}

impl From<StartType> for i32 {
    fn from(s: StartType) -> i32 {
        s as i32
    }
}

/// Aircraft placement parameters for [`XPlaneClient::place_aircraft`] and
/// [`XPlaneClient::load_and_place_aircraft`].
#[derive(Debug, Clone)]
pub struct PlacementConfig<'a> {
    pub start_type: StartType,
    /// aircraft slot index (0 = player aircraft, 1–19 = AI).
    pub aircraft_index: i32,
    /// airport ICAO ID, max 7 chars. Set to `""` when using [`StartType::SpecifyLatLonEle`].
    pub airport_id: &'a str,
    pub runway_index: i32,
    pub runway_direction: i32,
    /// latitude in degrees.
    pub latitude: f64,
    /// longitude in degrees.
    pub longitude: f64,
    /// elevation in meters.
    pub elevation: f64,
    /// true heading in degrees.
    pub heading: f64,
    /// speed in m/s.
    pub speed: f64,
}

impl Default for PlacementConfig<'_> {
    fn default() -> Self {
        PlacementConfig {
            start_type: StartType::SpecifyLatLonEle,
            aircraft_index: 0,
            airport_id: "",
            runway_index: 0,
            runway_direction: 0,
            latitude: 0.0,
            longitude: 0.0,
            elevation: 0.0,
            heading: 0.0,
            speed: 0.0,
        }
    }
}
