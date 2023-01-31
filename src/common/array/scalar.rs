mod impls;

pub trait Scalar: Sync + Send + Clone + std::fmt::Debug + 'static {}
