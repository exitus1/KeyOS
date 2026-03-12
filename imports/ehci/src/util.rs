use vcell::VolatileCell;

pub trait VolatileCellHelper<T> {
    fn change(&self, fm: impl FnOnce(&mut T));
}

impl<T: Copy> VolatileCellHelper<T> for VolatileCell<T> {
    fn change(&self, f: impl FnOnce(&mut T)) {
        let mut val = self.get();
        f(&mut val);
        self.set(val);
    }
}
