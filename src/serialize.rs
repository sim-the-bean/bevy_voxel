pub trait SerDePartialEq<T: ?Sized> {
    fn serde_eq(&self, other: &T) -> bool;
}

impl SerDePartialEq<Self> for f32 {
    fn serde_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl SerDePartialEq<Self> for i32 {
    fn serde_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T, U: SerDePartialEq<T>> SerDePartialEq<Option<T>> for Option<U> {
    fn serde_eq(&self, other: &Option<T>) -> bool {
        self.as_ref()
            .and_then(|this| other.as_ref().map(|other| this.serde_eq(other)))
            .unwrap_or(true)
    }
}
