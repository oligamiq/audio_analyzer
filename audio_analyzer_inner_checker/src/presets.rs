pub fn round(v: f64) -> f64 {
    v.round()
}
pub const fn pi() -> f64 {
    std::f64::consts::PI
}
pub fn cos(v: f64) -> f64 {
    v.cos()
}
pub fn log_10f64(v: f64) -> f64 {
    v.log10()
}
pub fn abs(v: f64) -> f64 {
    v.abs()
}
pub fn float<T: Into<f64>>(v: T) -> f64 {
    v.into()
}
